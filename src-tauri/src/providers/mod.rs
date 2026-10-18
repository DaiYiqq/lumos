//! Provider 层：DeepSeek / Ollama / none 的统一抽象与运行期选择。
//!
//! 选择顺序固定为 `deepseek → ollama → none`，且每次切换都会通过聊天事件
//! 明确告知前端，绝不静默切换。

pub mod deepseek;
pub mod none;
pub mod ollama;
pub mod prompt;
pub mod types;

use rusqlite::Connection;

use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::security;

pub use types::{
    ChatMessage, ChatRequest, ProviderError, ProviderEvent, ProviderId, ProviderStatus, Role,
};

/// Provider 流式输出通道。
pub type StreamSink = tokio::sync::mpsc::UnboundedSender<ProviderEvent>;

/// 统一 Provider 抽象。
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider 标识。
    fn id(&self) -> ProviderId;

    /// 是否具备调用条件（同步、便宜）。
    fn is_available(&self) -> bool;

    /// 流式生成；文本增量通过 `sink` 推送。
    ///
    /// # Errors
    ///
    /// 鉴权、限流、网络、模型不可用等错误按 [`ProviderError`] 分类返回。
    async fn stream_chat(
        &self,
        request: ChatRequest,
        sink: StreamSink,
    ) -> Result<(), ProviderError>;
}

/// Ollama 默认模型（可由设置覆盖）。
pub const DEFAULT_OLLAMA_MODEL: &str = "qwen2.5:0.5b";
/// DeepSeek 默认模型。
pub const DEFAULT_DEEPSEEK_MODEL: &str = "deepseek-chat";

/// 完整的 Provider 探测结果。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detection {
    /// 三个 Provider 的状态。
    pub statuses: Vec<ProviderStatus>,
    /// 自动选择结果。
    pub selected: ProviderId,
}

/// 探测 Ollama 是否可用（读取设置里的地址）。
///
/// # Errors
///
/// 读取设置失败时返回错误。
pub async fn probe_ollama(database: &Database) -> AppResult<(bool, String)> {
    let base = database.with_connection(ollama_base_url)?;
    Ok(ollama::OllamaProvider::probe(&base).await)
}

/// 探测全部 Provider（用于设置页展示；Ollama 会做一次健康检查）。
///
/// # Errors
///
/// 读取设置失败时返回错误。
pub async fn detect(database: &Database) -> AppResult<Detection> {
    let key_configured = security::load_deepseek_key().is_some();
    let key_source = security::deepseek_key_source();
    let deepseek_detail = match key_source {
        security::KeySource::Keyring => "已配置 API Key（系统凭据库）".to_string(),
        security::KeySource::Environment => "已配置 API Key（环境变量）".to_string(),
        security::KeySource::None => "未配置 API Key".to_string(),
    };

    let (ollama_base, ollama_model) = database.with_connection(|connection| {
        Ok((ollama_base_url(connection)?, ollama_model(connection)?))
    })?;
    let (ollama_available, ollama_detail) = ollama::OllamaProvider::probe(&ollama_base).await;
    let ollama_detail = if ollama_available {
        format!("{ollama_detail}（模型：{ollama_model}）")
    } else {
        ollama_detail
    };

    let selected = if key_configured {
        ProviderId::Deepseek
    } else if ollama_available {
        ProviderId::Ollama
    } else {
        ProviderId::None
    };

    let statuses = vec![
        ProviderStatus {
            id: ProviderId::Deepseek,
            available: key_configured,
            selected: selected == ProviderId::Deepseek,
            detail: deepseek_detail,
        },
        ProviderStatus {
            id: ProviderId::Ollama,
            available: ollama_available,
            selected: selected == ProviderId::Ollama,
            detail: ollama_detail,
        },
        ProviderStatus {
            id: ProviderId::None,
            available: true,
            selected: selected == ProviderId::None,
            detail: "离线关键词模式，永远可用".to_string(),
        },
    ];

    Ok(Detection { statuses, selected })
}

/// 构造指定 Provider。
///
/// # Errors
///
/// 读取设置失败时返回错误。
pub fn build_provider(database: &Database, id: ProviderId) -> AppResult<Box<dyn LlmProvider>> {
    match id {
        ProviderId::Deepseek => Ok(Box::new(deepseek::DeepSeekProvider::new(
            security::load_deepseek_key(),
        ))),
        ProviderId::Ollama => {
            let (base, model) = database.with_connection(|connection| {
                Ok((ollama_base_url(connection)?, ollama_model(connection)?))
            })?;
            Ok(Box::new(ollama::OllamaProvider::new(base, model, true)))
        }
        ProviderId::None => Ok(Box::new(none::NoneProvider)),
    }
}

/// 读取本次请求应使用的模型名。
///
/// # Errors
///
/// 读取设置失败时返回错误。
pub fn model_for(database: &Database, id: ProviderId) -> AppResult<String> {
    database.with_connection(|connection| match id {
        ProviderId::Deepseek => Ok(deepseek_model(connection)?),
        ProviderId::Ollama => Ok(ollama_model(connection)?),
        ProviderId::None => Ok(String::new()),
    })
}

/// Ollama 地址：环境变量优先（便于测试），其次设置，最后默认值。
///
/// # Errors
///
/// 读取设置失败时返回错误。
pub fn ollama_base_url(connection: &Connection) -> AppResult<String> {
    if let Ok(value) = std::env::var("OLLAMA_BASE_URL") {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }
    crate::db::repo::settings::get_or(connection, "ollama_base_url", ollama::DEFAULT_BASE_URL)
}

/// Ollama 模型名：环境变量优先，其次设置，最后默认值。
///
/// # Errors
///
/// 读取设置失败时返回错误。
pub fn ollama_model(connection: &Connection) -> AppResult<String> {
    if let Ok(value) = std::env::var("OLLAMA_MODEL") {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }
    crate::db::repo::settings::get_or(connection, "ollama_model", DEFAULT_OLLAMA_MODEL)
}

/// DeepSeek 模型名：环境变量优先，其次设置，最后默认值。
///
/// # Errors
///
/// 读取设置失败时返回错误。
pub fn deepseek_model(connection: &Connection) -> AppResult<String> {
    if let Ok(value) = std::env::var("DEEPSEEK_MODEL") {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }
    crate::db::repo::settings::get_or(connection, "deepseek_model", DEFAULT_DEEPSEEK_MODEL)
}

/// 把 Provider 错误转成命令层字符串（保持中文文案）。
#[must_use]
pub fn error_message(error: &ProviderError) -> String {
    error.user_message()
}

/// 允许写入的设置键白名单。
pub const SETTINGS_KEYS: &[&str] = &[
    "ollama_base_url",
    "ollama_model",
    "deepseek_model",
    "ui_language",
    "ui_theme",
];

/// 校验设置键是否允许写入。
///
/// # Errors
///
/// 键不在白名单内时返回错误。
pub fn validate_setting_key(key: &str) -> AppResult<()> {
    if SETTINGS_KEYS.contains(&key) {
        Ok(())
    } else {
        Err(AppError::InvalidInput(format!("不支持设置项：{key}")))
    }
}

/// 计算初始 Provider 顺序：deepseek → ollama → none。
#[must_use]
pub fn candidate_order(deepseek_configured: bool, ollama_available: bool) -> Vec<ProviderId> {
    let mut order = Vec::new();
    if deepseek_configured {
        order.push(ProviderId::Deepseek);
    }
    if ollama_available {
        order.push(ProviderId::Ollama);
    }
    order.push(ProviderId::None);
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_order_follows_priority() {
        assert_eq!(
            candidate_order(true, true),
            vec![ProviderId::Deepseek, ProviderId::Ollama, ProviderId::None]
        );
        assert_eq!(
            candidate_order(false, true),
            vec![ProviderId::Ollama, ProviderId::None]
        );
        assert_eq!(candidate_order(false, false), vec![ProviderId::None]);
    }

    #[test]
    fn settings_whitelist_rejects_unknown_keys() {
        assert!(validate_setting_key("ollama_base_url").is_ok());
        assert!(validate_setting_key("api_key").is_err());
    }
}
