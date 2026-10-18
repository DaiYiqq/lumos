//! Provider 层的公共类型与错误分类。

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 三种 Provider。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    /// DeepSeek 云端 API。
    Deepseek,
    /// 本地 Ollama。
    Ollama,
    /// 无模型时的规则降级（关键词检索）。
    None,
}

impl ProviderId {
    /// 稳定字符串。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deepseek => "deepseek",
            Self::Ollama => "ollama",
            Self::None => "none",
        }
    }
}

/// Provider 可用状态（供前端展示，不允许偷偷切换）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    /// Provider 标识。
    pub id: ProviderId,
    /// 是否可用。
    pub available: bool,
    /// 当前是否会被自动选择。
    pub selected: bool,
    /// 人类可读说明（例如「未配置 API Key」「Ollama 未运行」）。
    pub detail: String,
}

/// 聊天消息角色。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// 系统提示词。
    System,
    /// 用户。
    User,
    /// 助手。
    Assistant,
}

/// 一条聊天消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 角色。
    pub role: Role,
    /// 内容。
    pub content: String,
}

impl ChatMessage {
    /// 构造系统消息。
    #[must_use]
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    /// 构造用户消息。
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// 构造助手消息。
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// 一次流式聊天请求。
#[derive(Debug, Clone)]
pub struct ChatRequest {
    /// 模型名。
    pub model: String,
    /// 消息列表（已包含 system prompt）。
    pub messages: Vec<ChatMessage>,
    /// 采样温度。
    pub temperature: f32,
}

/// Provider 推给上层的流式事件。
#[derive(Debug, Clone)]
pub enum ProviderEvent {
    /// 增量文本。
    Delta(String),
}

/// Provider 错误分类：用于决定「重试 / 降级 / 提示用户」。
#[derive(Debug, Error)]
pub enum ProviderError {
    /// 未配置凭据。
    #[error("未配置 API Key")]
    MissingCredential,
    /// 鉴权失败。
    #[error("鉴权失败，请检查 API Key 是否正确")]
    Authentication,
    /// 触发限流。
    #[error("请求过于频繁，请稍后重试")]
    RateLimited,
    /// 模型不存在或没有权限。
    #[error("模型不可用，请检查模型名称")]
    ModelUnavailable,
    /// 网络不可达或超时。
    #[error("网络连接失败，请检查网络或代理设置")]
    Network,
    /// 服务端返回了无法解析的内容。
    #[error("模型返回了无法解析的内容")]
    MalformedResponse,
    /// 其他。
    #[error("{0}")]
    Other(String),
}

impl ProviderError {
    /// 是否适合降级到下一个 Provider。
    #[must_use]
    pub fn should_fallback(&self) -> bool {
        matches!(
            self,
            Self::MissingCredential
                | Self::Network
                | Self::ModelUnavailable
                | Self::RateLimited
                | Self::Authentication
        )
    }

    /// 面向用户的一句话。
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Other(message) => message.clone(),
            other => other.to_string(),
        }
    }
}

/// 根据 HTTP 状态码分类错误。
#[must_use]
pub fn classify_status(status: u16) -> ProviderError {
    match status {
        401 | 403 => ProviderError::Authentication,
        404 => ProviderError::ModelUnavailable,
        429 => ProviderError::RateLimited,
        500..=599 => ProviderError::Network,
        _ => ProviderError::Other(format!("模型服务返回 HTTP {status}")),
    }
}
