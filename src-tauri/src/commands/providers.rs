//! Provider 相关命令：API Key 管理（系统凭据库）、Provider 探测、设置读写。

use serde::Serialize;
use tauri::State;

use crate::db::repo::settings;
use crate::providers::{self, Detection};
use crate::security::{self, KeySource};
use crate::state::AppState;

/// API Key 状态（绝不返回 Key 本身）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyStatus {
    /// 是否已配置。
    pub configured: bool,
    /// 来源：`keyring` / `environment` / `none`。
    pub source: String,
}

/// 读取 DeepSeek Key 状态。
#[tauri::command]
pub fn api_key_status() -> ApiKeyStatus {
    let source = security::deepseek_key_source();
    ApiKeyStatus {
        configured: source != KeySource::None,
        source: source.as_str().to_string(),
    }
}

/// 保存 DeepSeek API Key 到系统凭据库。
///
/// # Errors
///
/// Key 为空或凭据库不可用时返回错误。
#[tauri::command]
pub fn api_key_set(key: String) -> Result<ApiKeyStatus, String> {
    security::store_deepseek_key(&key).map_err(|error| error.user_message())?;
    Ok(api_key_status())
}

/// 从系统凭据库删除 DeepSeek API Key。
///
/// # Errors
///
/// 凭据库不可用时返回错误。
#[tauri::command]
pub fn api_key_clear() -> Result<ApiKeyStatus, String> {
    security::clear_deepseek_key().map_err(|error| error.user_message())?;
    Ok(api_key_status())
}

/// 探测三个 Provider 的可用状态。
///
/// # Errors
///
/// 读取设置失败时返回错误。
#[tauri::command]
pub async fn providers_detect(state: State<'_, AppState>) -> Result<Detection, String> {
    providers::detect(&state.db)
        .await
        .map_err(|error| error.user_message())
}

/// 读取一个设置项。
///
/// # Errors
///
/// 键不合法或数据库查询失败时返回错误。
#[tauri::command]
pub fn settings_get(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    providers::validate_setting_key(&key).map_err(|error| error.user_message())?;
    state
        .db
        .with_connection(|connection| settings::get(connection, &key))
        .map_err(|error| error.user_message())
}

/// 写入一个设置项。
///
/// # Errors
///
/// 键不合法或数据库写入失败时返回错误。
#[tauri::command]
pub fn settings_set(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    providers::validate_setting_key(&key).map_err(|error| error.user_message())?;
    state
        .db
        .with_connection(|connection| settings::set(connection, &key, &value))
        .map_err(|error| error.user_message())
}
