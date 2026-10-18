//! 应用设置仓储：只存非敏感配置（API Key 走系统凭据库）。

use rusqlite::{Connection, OptionalExtension};

use crate::error::AppResult;

/// 读取一个设置项。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
pub fn get(connection: &Connection, key: &str) -> AppResult<Option<String>> {
    let value = connection
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()?;
    Ok(value)
}

/// 写入（或覆盖）一个设置项。
///
/// # Errors
///
/// 数据库写入失败时返回错误。
pub fn set(connection: &Connection, key: &str, value: &str) -> AppResult<()> {
    connection.execute(
        "INSERT INTO settings (key, value, updated_at) \
         VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

/// 读取设置，找不到时返回默认值。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
pub fn get_or(connection: &Connection, key: &str, default: &str) -> AppResult<String> {
    Ok(get(connection, key)?.unwrap_or_else(|| default.to_string()))
}
