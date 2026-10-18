//! 离线降级 Provider：不进行生成，只提供关键词检索结果。
//!
//! 真正的离线回答由 `commands/chat.rs` 组装（直接把检索片段返回给用户），
//! 这里保留空实现，是为了让 Provider 抽象保持完整。

use async_trait::async_trait;

use crate::providers::types::{ChatRequest, ProviderError, ProviderEvent, ProviderId};
use crate::providers::{LlmProvider, StreamSink};

/// 规则降级的占位 Provider。
pub struct NoneProvider;

#[async_trait]
impl LlmProvider for NoneProvider {
    fn id(&self) -> ProviderId {
        ProviderId::None
    }

    fn is_available(&self) -> bool {
        true
    }

    async fn stream_chat(
        &self,
        _request: ChatRequest,
        _sink: StreamSink,
    ) -> Result<(), ProviderError> {
        let _ = ProviderEvent::Delta(String::new());
        Err(ProviderError::Other(
            "离线模式不进行模型生成，直接返回检索片段".to_string(),
        ))
    }
}
