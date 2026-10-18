//! Ollama Provider（本地 `POST /api/chat` 流式接口）。

use async_trait::async_trait;
use futures_util::StreamExt;
use serde_json::json;

use crate::providers::types::{ChatRequest, ProviderError, ProviderEvent, ProviderId};
use crate::providers::{LlmProvider, StreamSink};

/// 默认本地地址。
pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11434";

/// Ollama Provider。
pub struct OllamaProvider {
    base_url: String,
    default_model: String,
    available: bool,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// 创建 Provider。
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        default_model: impl Into<String>,
        available: bool,
    ) -> Self {
        let base_url = base_url.into();
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            base_url,
            default_model: default_model.into(),
            available,
            client: reqwest::Client::new(),
        }
    }

    /// 探测 Ollama 是否可用，返回（是否可用, 说明）。
    pub async fn probe(base_url: &str) -> (bool, String) {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(1200))
            .build()
        {
            Ok(client) => client,
            Err(_) => return (false, "无法创建网络客户端".to_string()),
        };
        let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => (true, "Ollama 正在运行".to_string()),
            Ok(_) => (false, "Ollama 有响应但状态异常".to_string()),
            Err(_) => (false, "未检测到 Ollama（需要本地安装并启动）".to_string()),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Ollama
    }

    fn is_available(&self) -> bool {
        self.available
    }

    async fn stream_chat(
        &self,
        request: ChatRequest,
        sink: StreamSink,
    ) -> Result<(), ProviderError> {
        let model = if request.model.trim().is_empty() {
            self.default_model.clone()
        } else {
            request.model.clone()
        };
        if model.trim().is_empty() {
            return Err(ProviderError::ModelUnavailable);
        }

        let url = format!("{}/api/chat", self.base_url);
        let body = json!({
            "model": model,
            "messages": request.messages,
            "stream": true,
            "options": { "temperature": request.temperature },
        });

        let response = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|_| ProviderError::Network)?;

        if !response.status().is_success() {
            return Err(crate::providers::types::classify_status(
                response.status().as_u16(),
            ));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| ProviderError::Network)?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(position) = buffer.find('\n') {
                let line = buffer[..position].trim().to_string();
                buffer.drain(..position + 1);
                if line.is_empty() {
                    continue;
                }
                match parse_line(&line)? {
                    OllamaEvent::Delta(text) => {
                        let _ = sink.send(ProviderEvent::Delta(text));
                    }
                    OllamaEvent::Done => return Ok(()),
                }
            }
        }

        Ok(())
    }
}

/// 单行 NDJSON 的解析结果。
enum OllamaEvent {
    /// 文本增量。
    Delta(String),
    /// 流结束。
    Done,
}

/// 解析 Ollama 的一行流式响应。
fn parse_line(line: &str) -> Result<OllamaEvent, ProviderError> {
    let value: serde_json::Value =
        serde_json::from_str(line).map_err(|_| ProviderError::MalformedResponse)?;

    if let Some(error) = value.get("error").and_then(serde_json::Value::as_str) {
        return Err(ProviderError::Other(format!("Ollama 返回错误：{error}")));
    }

    if value
        .get("done")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(OllamaEvent::Done);
    }

    let content = value
        .pointer("/message/content")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    Ok(OllamaEvent::Delta(content.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stream_chunk() {
        let line =
            r#"{"model":"qwen","message":{"role":"assistant","content":"你好"},"done":false}"#;
        match parse_line(line).expect("应能解析") {
            OllamaEvent::Delta(text) => assert_eq!(text, "你好"),
            OllamaEvent::Done => panic!("不应结束"),
        }
    }

    #[test]
    fn parses_done_flag() {
        assert!(matches!(
            parse_line(r#"{"done":true}"#).expect("应能解析"),
            OllamaEvent::Done
        ));
    }

    #[test]
    fn surfaces_ollama_error_payload() {
        let error = parse_line(r#"{"error":"model not found"}"#);
        assert!(matches!(error, Err(ProviderError::Other(_))));
    }

    /// 真实 Ollama（或兼容 mock 服务）流式调用。
    ///
    /// 运行方式：
    /// `OLLAMA_E2E=1 OLLAMA_BASE_URL=http://127.0.0.1:11435 OLLAMA_MODEL=mock \
    ///  cargo test real_ollama_streams -- --ignored --nocapture`
    #[tokio::test]
    #[ignore = "需要本地 Ollama 或 mock 服务"]
    async fn real_ollama_streams() {
        use crate::providers::ChatMessage;

        if std::env::var("OLLAMA_E2E").is_err() {
            eprintln!("跳过：未设置 OLLAMA_E2E");
            return;
        }
        let base =
            std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let (up, detail) = OllamaProvider::probe(&base).await;
        assert!(up, "服务应可用：{detail}");

        let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "mock".to_string());
        let provider = OllamaProvider::new(base, model, true);
        let request = ChatRequest {
            model: String::new(),
            messages: vec![ChatMessage::user("你好")],
            temperature: 0.0,
        };
        let (sink, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let worker = tokio::spawn(async move { provider.stream_chat(request, sink).await });

        let mut text = String::new();
        while let Some(ProviderEvent::Delta(delta)) = receiver.recv().await {
            text.push_str(&delta);
        }
        let result = worker.await.expect("任务不应 panic");
        assert!(result.is_ok(), "调用应成功: {result:?}");
        assert!(!text.trim().is_empty(), "应收到流式文本");
        eprintln!("Ollama 流式响应：{text}");
    }
}
