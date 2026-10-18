//! 敏感凭据管理。
//!
//! API Key 只存放在操作系统凭据库（Windows 凭据管理器 / macOS Keychain /
//! Linux Secret Service），绝不写入数据库、配置文件或日志。
//! 开发/CI 场景允许通过进程环境变量 `DEEPSEEK_API_KEY` 注入，仅存在内存中。

use crate::error::{AppError, AppResult};

/// 凭据库中的服务名（与应用标识一致）。
const SERVICE: &str = "com.daiyi.lumos";
/// DeepSeek Key 的账号名。
const DEEPSEEK_ACCOUNT: &str = "deepseek";

/// Key 的来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// 系统凭据库。
    Keyring,
    /// 环境变量（开发/CI）。
    Environment,
    /// 未配置。
    None,
}

impl KeySource {
    /// 稳定的字符串标识（给前端展示）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Keyring => "keyring",
            Self::Environment => "environment",
            Self::None => "none",
        }
    }
}

/// 读取 DeepSeek API Key（不打印、不返回给前端）。
#[must_use]
pub fn load_deepseek_key() -> Option<String> {
    if let Some(secret) = read_keyring_secret() {
        return Some(secret);
    }
    environment_key()
}

/// 当前 Key 的来源。
#[must_use]
pub fn deepseek_key_source() -> KeySource {
    if read_keyring_secret().is_some() {
        return KeySource::Keyring;
    }
    if environment_key().is_some() {
        return KeySource::Environment;
    }
    KeySource::None
}

/// 把 Key 写入系统凭据库（覆盖旧值）。
///
/// # Errors
///
/// Key 为空、凭据库不可用或写入失败时返回错误。
pub fn store_deepseek_key(key: &str) -> AppResult<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("API Key 不能为空".to_string()));
    }
    let entry = keyring::Entry::new(SERVICE, DEEPSEEK_ACCOUNT)
        .map_err(|error| AppError::Internal(format!("无法访问系统凭据库：{error}")))?;
    entry
        .set_password(trimmed)
        .map_err(|error| AppError::Internal(format!("写入系统凭据库失败：{error}")))
}

/// 删除系统凭据库中的 Key。
///
/// # Errors
///
/// 凭据库不可用或删除失败时返回错误；本来就不存在时视为成功。
pub fn clear_deepseek_key() -> AppResult<()> {
    let entry = keyring::Entry::new(SERVICE, DEEPSEEK_ACCOUNT)
        .map_err(|error| AppError::Internal(format!("无法访问系统凭据库：{error}")))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(AppError::Internal(format!("删除系统凭据失败：{error}"))),
    }
}

/// 读取系统凭据库里的 Key。
fn read_keyring_secret() -> Option<String> {
    let entry = keyring::Entry::new(SERVICE, DEEPSEEK_ACCOUNT).ok()?;
    let secret = entry.get_password().ok()?;
    let trimmed = secret.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// 读取环境变量中的 Key（仅开发/CI）。
fn environment_key() -> Option<String> {
    std::env::var("DEEPSEEK_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
