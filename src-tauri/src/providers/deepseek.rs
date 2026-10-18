//! DeepSeek Provider（OpenAI 兼容的 `/chat/completions` 流式接口）。

use async_trait::async_trait;
use futures_util::StreamExt;
use serde_json::json;

use crate::providers::types::{
    ChatRequest, ProviderError, ProviderEvent, ProviderId, classify_status,
};
use crate::providers::{LlmProvider, StreamSink};

/// DeepSeek 官方默认基址。
const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";
/// 默认模型。
const DEFAULT_MODEL: &str = "deepseek-chat";

/// DeepSeek Provider。
pub struct DeepSeekProvider {
    api_key: Option<String>,
    base_url: String,
    default_model: String,
    client: reqwest::Client,
}

impl DeepSeekProvider {
    /// 创建 Provider；`api_key` 为 `None` 时视为不可用。
    #[must_use]
    pub fn new(api_key: Option<String>) -> Self {
        let base_url = std::env::var("DEEPSEEK_BASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let default_model = std::env::var("DEEPSEEK_MODEL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        Self {
            api_key,
            base_url,
            default_model,
            client: reqwest::Client::new(),
        }
    }

    /// 默认模型名。
    #[must_use]
    pub fn default_model(&self) -> &str {
        &self.default_model
    }
}

#[async_trait]
impl LlmProvider for DeepSeekProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Deepseek
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    async fn stream_chat(
        &self,
        request: ChatRequest,
        sink: StreamSink,
    ) -> Result<(), ProviderError> {
        let api_key = self
            .api_key
            .as_deref()
            .ok_or(ProviderError::MissingCredential)?;
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let model = if request.model.trim().is_empty() {
            self.default_model.clone()
        } else {
            request.model.clone()
        };
        let body = json!({
            "model": model,
            "messages": request.messages,
            "stream": true,
            "temperature": request.temperature,
        });

        let response = self
            .client
            .post(url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|_| ProviderError::Network)?;

        if !response.status().is_success() {
            return Err(classify_status(response.status().as_u16()));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| ProviderError::Network)?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(position) = buffer.find("\n\n") {
                let event = buffer[..position].to_string();
                buffer.drain(..position + 2);

                match parse_sse_event(&event)? {
                    SseEvent::Delta(text) => {
                        let _ = sink.send(ProviderEvent::Delta(text));
                    }
                    SseEvent::Done => return Ok(()),
                    SseEvent::Ignore => {}
                }
            }
        }

        Ok(())
    }
}

/// 单个 SSE 事件的解析结果。
enum SseEvent {
    /// 文本增量。
    Delta(String),
    /// 流结束。
    Done,
    /// 无需处理（心跳、空白等）。
    Ignore,
}

/// 解析一个 SSE 事件块。
fn parse_sse_event(event: &str) -> Result<SseEvent, ProviderError> {
    for line in event.lines() {
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() {
            continue;
        }
        if data == "[DONE]" {
            return Ok(SseEvent::Done);
        }

        let value: serde_json::Value =
            serde_json::from_str(data).map_err(|_| ProviderError::MalformedResponse)?;
        let delta = value
            .pointer("/choices/0/delta/content")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if delta.is_empty() {
            return Ok(SseEvent::Ignore);
        }
        return Ok(SseEvent::Delta(delta.to_string()));
    }
    Ok(SseEvent::Ignore)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_delta_event() {
        let event = "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}";
        match parse_sse_event(event).expect("应能解析") {
            SseEvent::Delta(text) => assert_eq!(text, "你好"),
            _ => panic!("应为 delta 事件"),
        }
    }

    #[test]
    fn parses_done_event() {
        assert!(matches!(
            parse_sse_event("data: [DONE]").expect("应能解析"),
            SseEvent::Done
        ));
    }

    #[test]
    fn ignores_heartbeat_and_empty_events() {
        assert!(matches!(
            parse_sse_event(": keep-alive").expect("应能解析"),
            SseEvent::Ignore
        ));
        assert!(matches!(
            parse_sse_event("").expect("应能解析"),
            SseEvent::Ignore
        ));
    }

    #[test]
    fn malformed_json_is_reported() {
        assert!(matches!(
            parse_sse_event("data: {oops}"),
            Err(ProviderError::MalformedResponse)
        ));
    }

    #[test]
    fn classifies_http_status() {
        assert!(matches!(
            classify_status(401),
            ProviderError::Authentication
        ));
        assert!(matches!(classify_status(429), ProviderError::RateLimited));
        assert!(matches!(
            classify_status(404),
            ProviderError::ModelUnavailable
        ));
        assert!(classify_status(503).should_fallback());
    }

    /// 真实 DeepSeek 流式调用（需要 DEEPSEEK_API_KEY）。
    ///
    /// 运行方式：
    /// `DEEPSEEK_API_KEY=... cargo test real_deepseek_streams -- --ignored --nocapture`
    #[tokio::test]
    #[ignore = "需要真实 DEEPSEEK_API_KEY 环境变量"]
    async fn real_deepseek_streams() {
        use crate::providers::ChatMessage;

        let Ok(key) = std::env::var("DEEPSEEK_API_KEY") else {
            eprintln!("跳过：未设置 DEEPSEEK_API_KEY");
            return;
        };

        let provider = DeepSeekProvider::new(Some(key.trim().to_string()));
        let request = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                ChatMessage::system("你是一个测试助手，请只回复四个字：流式正常"),
                ChatMessage::user("请回复"),
            ],
            temperature: 0.0,
        };
        let (sink, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let worker = tokio::spawn(async move { provider.stream_chat(request, sink).await });

        let mut text = String::new();
        while let Some(ProviderEvent::Delta(delta)) = receiver.recv().await {
            text.push_str(&delta);
        }
        let result = worker.await.expect("任务不应 panic");
        assert!(result.is_ok(), "真实调用应成功: {result:?}");
        assert!(!text.trim().is_empty(), "应收到流式文本");
        eprintln!("真实 DeepSeek 流式响应：{text}");
    }
}
