//! 问答命令：检索 → 组装 Prompt → Provider 流式生成 → 推送引用与增量文本。
//!
//! 前端通过 Tauri Channel 接收 [`ChatEvent`]；Provider 的切换（含降级）
//! 都会发送明确的 `notice` 事件，绝不静默改变模式。

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::State;
use tauri::ipc::Channel;

use crate::providers::{self, ChatMessage, ChatRequest, ProviderEvent, ProviderId, prompt};
use crate::retrieval::{self, SearchHit, SearchMode};
use crate::security;
use crate::state::AppState;

/// 一次问答中推送给前端的事件。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatEvent {
    /// 当前使用的 Provider（前端据此显示模式徽标）。
    Provider {
        /// Provider 标识。
        provider: String,
        /// 展示名。
        label: String,
    },
    /// 提示信息（降级、模型加载等）。
    Notice {
        /// 提示文案。
        message: String,
    },
    /// 引用列表（先生成，再开始流式输出）。
    Citations {
        /// 引用详情。
        citations: Vec<Citation>,
    },
    /// 文本增量。
    Delta {
        /// 增量文本。
        text: String,
    },
    /// 生成完成。
    Done {
        /// Provider 标识。
        provider: String,
    },
    /// 生成失败。
    Error {
        /// 面向用户的错误。
        message: String,
    },
}

/// 与回答一一对应的引用（编号从 1 开始）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    /// 引用编号（与正文里的 `[n]` 对应）。
    pub index: usize,
    /// 分块 ID。
    pub chunk_id: i64,
    /// 文档 ID。
    pub document_id: String,
    /// 文档标题。
    pub document_title: String,
    /// 文档路径。
    pub document_path: String,
    /// 页码。
    pub page_no: Option<i64>,
    /// 章节路径。
    pub section_path: Option<String>,
    /// 原文片段。
    pub content: String,
    /// 融合检索分数。
    pub score: f64,
}

/// 流式问答。
///
/// # Errors
///
/// 问题为空或检索失败时返回错误；模型调用错误通过 [`ChatEvent::Error`] 推送。
#[tauri::command]
pub async fn chat_stream(
    state: State<'_, AppState>,
    library_id: String,
    question: String,
    history: Option<Vec<ChatMessage>>,
    channel: Channel<ChatEvent>,
) -> Result<(), String> {
    let question = question.trim().to_string();
    if question.is_empty() {
        return Err("请输入问题".to_string());
    }

    let response = retrieval::search(
        &state.db,
        &state.embedder,
        &library_id,
        &question,
        SearchMode::Hybrid,
        6,
    )
    .map_err(|error| error.user_message())?;
    let hits = response.hits;

    let citations: Vec<Citation> = hits
        .iter()
        .enumerate()
        .map(|(index, hit)| citation_from_hit(index + 1, hit))
        .collect();
    let _ = channel.send(ChatEvent::Citations { citations });

    if hits.is_empty() {
        let _ = channel.send(ChatEvent::Provider {
            provider: "none".to_string(),
            label: "离线关键词模式".to_string(),
        });
        let _ = channel.send(ChatEvent::Delta {
            text: prompt::offline_answer(&[]),
        });
        let _ = channel.send(ChatEvent::Done {
            provider: "none".to_string(),
        });
        return Ok(());
    }

    let context = prompt::build_context(&hits);
    let messages = prompt::build_messages(&context, history.as_deref(), &question);

    let key_configured = security::load_deepseek_key().is_some();
    let mut queue: VecDeque<ProviderId> = VecDeque::new();
    let mut ollama_probed = false;

    if key_configured {
        queue.push_back(ProviderId::Deepseek);
    } else {
        ollama_probed = true;
        match providers::probe_ollama(&state.db).await {
            Ok((true, _)) => queue.push_back(ProviderId::Ollama),
            Ok((false, detail)) => {
                let _ = channel.send(ChatEvent::Notice {
                    message: format!("{detail}，将使用离线关键词模式"),
                });
            }
            Err(_) => {}
        }
    }
    queue.push_back(ProviderId::None);

    while let Some(candidate) = queue.pop_front() {
        if candidate == ProviderId::None {
            // DeepSeek 失败后再探测一次 Ollama
            if !ollama_probed {
                ollama_probed = true;
                if let Ok((true, _)) = providers::probe_ollama(&state.db).await {
                    queue.push_front(ProviderId::Ollama);
                    continue;
                }
            }

            let _ = channel.send(ChatEvent::Provider {
                provider: "none".to_string(),
                label: "离线关键词模式".to_string(),
            });
            let _ = channel.send(ChatEvent::Delta {
                text: prompt::offline_answer(&hits),
            });
            let _ = channel.send(ChatEvent::Done {
                provider: "none".to_string(),
            });
            return Ok(());
        }

        let provider = providers::build_provider(&state.db, candidate)
            .map_err(|error| error.user_message())?;
        if !provider.is_available() {
            let _ = channel.send(ChatEvent::Notice {
                message: format!("{} 当前不可用，已跳过", provider_label(candidate)),
            });
            continue;
        }

        let model = providers::model_for(&state.db, candidate).unwrap_or_default();
        let _ = channel.send(ChatEvent::Provider {
            provider: candidate.as_str().to_string(),
            label: provider_label(candidate).to_string(),
        });

        let (sink, mut receiver) = tokio::sync::mpsc::unbounded_channel::<ProviderEvent>();
        let emitted = Arc::new(AtomicBool::new(false));
        let forward_channel = channel.clone();
        let forward_emitted = Arc::clone(&emitted);
        let forward = tauri::async_runtime::spawn(async move {
            while let Some(ProviderEvent::Delta(text)) = receiver.recv().await {
                if text.is_empty() {
                    continue;
                }
                forward_emitted.store(true, Ordering::Relaxed);
                let _ = forward_channel.send(ChatEvent::Delta { text });
            }
        });

        let request = ChatRequest {
            model,
            messages: messages.clone(),
            temperature: 0.3,
        };
        let result = provider.stream_chat(request, sink).await;
        let _ = forward.await;

        match result {
            Ok(()) => {
                let _ = channel.send(ChatEvent::Done {
                    provider: candidate.as_str().to_string(),
                });
                return Ok(());
            }
            Err(error) => {
                if !emitted.load(Ordering::Relaxed) && error.should_fallback() {
                    let _ = channel.send(ChatEvent::Notice {
                        message: format!(
                            "{} 调用失败：{}，正在尝试降级…",
                            provider_label(candidate),
                            error.user_message()
                        ),
                    });
                    continue;
                }
                let _ = channel.send(ChatEvent::Error {
                    message: error.user_message(),
                });
                return Ok(());
            }
        }
    }

    Err("没有可用的模型服务，请检查网络或 Ollama 状态".to_string())
}

/// 把检索命中映射为引用。
fn citation_from_hit(index: usize, hit: &SearchHit) -> Citation {
    Citation {
        index,
        chunk_id: hit.chunk_id,
        document_id: hit.document_id.clone(),
        document_title: hit.document_title.clone(),
        document_path: hit.document_path.clone(),
        page_no: hit.page_no,
        section_path: hit.section_path.clone(),
        content: hit.content.clone(),
        score: hit.score,
    }
}

/// Provider 的中文展示名。
fn provider_label(id: ProviderId) -> &'static str {
    match id {
        ProviderId::Deepseek => "DeepSeek",
        ProviderId::Ollama => "Ollama",
        ProviderId::None => "离线关键词模式",
    }
}

/// 非流式问答结果（供 CLI / 自动化验收与未来的快捷调用使用）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAnswer {
    /// 实际使用的 Provider。
    pub provider: String,
    /// 完整回答文本。
    pub answer: String,
    /// 引用列表。
    pub citations: Vec<Citation>,
}

/// 非流式问答：内部走与流式相同的检索、Prompt 与 Provider 降级链路。
///
/// # Errors
///
/// 问题为空、检索失败或所有 Provider 都不可用时返回错误。
#[tauri::command]
pub async fn chat_complete(
    state: State<'_, AppState>,
    library_id: String,
    question: String,
) -> Result<ChatAnswer, String> {
    let question = question.trim().to_string();
    if question.is_empty() {
        return Err("请输入问题".to_string());
    }

    let response = retrieval::search(
        &state.db,
        &state.embedder,
        &library_id,
        &question,
        SearchMode::Hybrid,
        6,
    )
    .map_err(|error| error.user_message())?;
    let hits = response.hits;
    let citations: Vec<Citation> = hits
        .iter()
        .enumerate()
        .map(|(index, hit)| citation_from_hit(index + 1, hit))
        .collect();

    let key_configured = security::load_deepseek_key().is_some();
    let (ollama_available, _) = providers::probe_ollama(&state.db)
        .await
        .unwrap_or((false, String::new()));
    let order = providers::candidate_order(key_configured, ollama_available);

    if hits.is_empty() {
        return Ok(ChatAnswer {
            provider: "none".to_string(),
            answer: prompt::offline_answer(&[]),
            citations,
        });
    }

    let context = prompt::build_context(&hits);
    let messages = prompt::build_messages(&context, None, &question);

    for candidate in order {
        if candidate == ProviderId::None {
            return Ok(ChatAnswer {
                provider: "none".to_string(),
                answer: prompt::offline_answer(&hits),
                citations,
            });
        }

        let provider = providers::build_provider(&state.db, candidate)
            .map_err(|error| error.user_message())?;
        if !provider.is_available() {
            continue;
        }
        let model = providers::model_for(&state.db, candidate).unwrap_or_default();
        let (sink, mut receiver) = tokio::sync::mpsc::unbounded_channel::<ProviderEvent>();
        let worker = provider.stream_chat(
            ChatRequest {
                model,
                messages: messages.clone(),
                temperature: 0.3,
            },
            sink,
        );
        let mut answer = String::new();
        let collect = async {
            while let Some(ProviderEvent::Delta(delta)) = receiver.recv().await {
                answer.push_str(&delta);
            }
        };
        let (result, ()) = tokio::join!(worker, collect);
        match result {
            Ok(()) if !answer.trim().is_empty() => {
                return Ok(ChatAnswer {
                    provider: candidate.as_str().to_string(),
                    answer,
                    citations,
                });
            }
            Ok(()) => continue,
            Err(_) => continue,
        }
    }

    Ok(ChatAnswer {
        provider: "none".to_string(),
        answer: prompt::offline_answer(&hits),
        citations,
    })
}
