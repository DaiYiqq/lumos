//! 本地文本嵌入服务（fastembed + BGE-small-zh-v1.5）。
//!
//! 设计要点：
//!   * 模型在后台线程加载，不阻塞窗口启动；首次使用会从 HuggingFace 下载约 100MB。
//!   * 模型不可用时，检索降级为纯关键词（FTS5），应用仍然可用。
//!   * 模型缓存在应用数据目录的 `models/` 下，卸载时随数据目录一起清理。

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use fastembed::{InitOptionsUserDefined, Pooling, TextEmbedding, UserDefinedEmbeddingModel};
use serde::Serialize;

use crate::error::{AppError, AppResult};

mod model_store;

/// 嵌入维度（BGE-small-zh-v1.5 输出 512 维）。
pub const EMBEDDING_DIM: usize = 512;

/// 模型标识，写入数据库用于将来判断是否需要重建向量。
pub const MODEL_ID: &str = "BAAI/bge-small-zh-v1.5";

/// 单次推理的批大小。
const BATCH_SIZE: usize = 32;

/// BGE 系列模型在「检索查询」侧推荐使用的指令前缀（官方用法）。
const QUERY_INSTRUCTION: &str = "为这个句子生成表示以用于检索相关文章：";

/// 嵌入服务状态（前端展示用）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingStatus {
    /// `loading` / `ready` / `failed`。
    pub state: String,
    /// 模型标识。
    pub model: String,
    /// 向量维度。
    pub dimension: usize,
    /// 失败原因（面向日志，不直接展示给用户）。
    pub message: Option<String>,
}

/// 内部状态机。
enum Inner {
    Loading,
    Ready(Box<TextEmbedding>),
    Failed(String),
}

/// 线程安全的嵌入服务。
pub struct EmbeddingService {
    inner: Mutex<Inner>,
    cache_dir: PathBuf,
}

impl EmbeddingService {
    /// 创建服务并进入加载状态。
    #[must_use]
    pub fn new(cache_dir: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner::Loading),
            cache_dir,
        })
    }

    /// 在后台线程加载模型（含首次下载）。
    pub fn warmup(self: &Arc<Self>) {
        let service = Arc::clone(self);
        std::thread::spawn(move || service.load_model());
    }

    /// 同步加载模型。失败时记录原因，调用方通过 [`Self::status`] 查询。
    fn load_model(&self) {
        let next_state = match self.create_model() {
            Ok(model) => Inner::Ready(Box::new(model)),
            Err(error) => Inner::Failed(error.user_message()),
        };
        if let Ok(mut guard) = self.inner.lock() {
            *guard = next_state;
        }
    }

    /// 下载（或复用缓存）模型文件并从本地文件构建 ONNX 会话。
    fn create_model(&self) -> AppResult<TextEmbedding> {
        let files = model_store::ensure_model_files(&self.cache_dir)?;
        let tokenizer_files = fastembed::TokenizerFiles {
            tokenizer_file: std::fs::read(&files.tokenizer)?,
            config_file: std::fs::read(&files.config)?,
            special_tokens_map_file: std::fs::read(&files.special_tokens_map)?,
            tokenizer_config_file: std::fs::read(&files.tokenizer_config)?,
        };
        let model = UserDefinedEmbeddingModel::new(std::fs::read(&files.onnx)?, tokenizer_files)
            .with_pooling(Pooling::Cls);
        let options = InitOptionsUserDefined::new().with_max_length(512);

        TextEmbedding::try_new_from_user_defined(model, options)
            .map_err(|error| AppError::Internal(format!("嵌入模型加载失败：{error}")))
    }

    /// 当前状态。
    #[must_use]
    pub fn status(&self) -> EmbeddingStatus {
        match self.inner.lock() {
            Ok(guard) => match &*guard {
                Inner::Loading => EmbeddingStatus {
                    state: "loading".to_string(),
                    model: MODEL_ID.to_string(),
                    dimension: EMBEDDING_DIM,
                    message: None,
                },
                Inner::Ready(_) => EmbeddingStatus {
                    state: "ready".to_string(),
                    model: MODEL_ID.to_string(),
                    dimension: EMBEDDING_DIM,
                    message: None,
                },
                Inner::Failed(message) => EmbeddingStatus {
                    state: "failed".to_string(),
                    model: MODEL_ID.to_string(),
                    dimension: EMBEDDING_DIM,
                    message: Some(message.clone()),
                },
            },
            Err(_) => EmbeddingStatus {
                state: "failed".to_string(),
                model: MODEL_ID.to_string(),
                dimension: EMBEDDING_DIM,
                message: Some("嵌入服务状态锁已中毒".to_string()),
            },
        }
    }

    /// 模型是否已可用。
    #[must_use]
    pub fn is_ready(&self) -> bool {
        matches!(self.inner.lock(), Ok(guard) if matches!(&*guard, Inner::Ready(_)))
    }

    /// 模型是否加载失败。
    #[must_use]
    pub fn has_failed(&self) -> bool {
        matches!(self.inner.lock(), Ok(guard) if matches!(&*guard, Inner::Failed(_)))
    }

    /// 批量生成文档向量。
    ///
    /// # Errors
    ///
    /// 模型仍在加载、已失败或推理出错时返回面向用户的错误。
    pub fn embed_documents(&self, texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| AppError::Internal("嵌入服务锁已中毒".to_string()))?;
        match &mut *guard {
            Inner::Ready(model) => {
                let batch: Vec<&str> = texts.iter().map(String::as_str).collect();
                model
                    .embed(batch, Some(BATCH_SIZE))
                    .map_err(|error| AppError::InvalidInput(format!("本地嵌入计算失败：{error}")))
            }
            Inner::Loading => Err(AppError::InvalidInput(
                "本地嵌入模型正在加载，请稍后重试".to_string(),
            )),
            Inner::Failed(_) => Err(AppError::InvalidInput(
                "本地嵌入模型不可用，当前只能使用关键词检索".to_string(),
            )),
        }
    }

    /// 生成单条查询向量（自动加上 BGE 推荐的检索指令前缀）。
    ///
    /// # Errors
    ///
    /// 同 [`Self::embed_documents`]。
    pub fn embed_query(&self, query: &str) -> AppResult<Vec<f32>> {
        let text = format!("{QUERY_INSTRUCTION}{query}");
        let mut vectors = self.embed_documents(&[text])?;
        vectors
            .pop()
            .ok_or_else(|| AppError::Internal("嵌入服务返回了空向量".to_string()))
    }
}
