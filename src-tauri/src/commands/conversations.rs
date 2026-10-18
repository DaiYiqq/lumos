//! 对话历史相关命令。

use serde::Serialize;
use tauri::State;

use crate::db::models::{Conversation, ConversationSummary, StoredMessage};
use crate::db::repo::conversations as repo;
use crate::error::AppError;
use crate::state::AppState;

/// 会话详情：基本信息 + 消息列表。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationDetail {
    /// 会话信息。
    pub conversation: Conversation,
    /// 消息列表（按时间顺序）。
    pub messages: Vec<StoredMessage>,
}

/// 把领域错误转成前端可展示文案。
fn to_message(error: AppError) -> String {
    error.user_message()
}

/// 列出知识库下的会话。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
#[tauri::command]
pub fn conversation_list(
    state: State<'_, AppState>,
    library_id: String,
) -> Result<Vec<ConversationSummary>, String> {
    state
        .db
        .with_connection(|connection| repo::list_conversations(connection, &library_id))
        .map_err(to_message)
}

/// 创建会话（标题缺省为「新对话」）。
///
/// # Errors
///
/// 数据库写入失败时返回错误。
#[tauri::command]
pub fn conversation_create(
    state: State<'_, AppState>,
    library_id: String,
    title: Option<String>,
) -> Result<Conversation, String> {
    let title = title.unwrap_or_else(|| "新对话".to_string());
    state
        .db
        .with_connection(|connection| repo::create_conversation(connection, &library_id, &title))
        .map_err(to_message)
}

/// 读取会话详情（含消息）。
///
/// # Errors
///
/// 会话不存在或数据库查询失败时返回错误。
#[tauri::command]
pub fn conversation_get(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConversationDetail, String> {
    state
        .db
        .with_connection(|connection| {
            let conversation = repo::get_conversation(connection, &id)?
                .ok_or_else(|| AppError::InvalidInput("会话不存在或已被删除".to_string()))?;
            let messages = repo::list_messages(connection, &id)?;
            Ok(ConversationDetail {
                conversation,
                messages,
            })
        })
        .map_err(to_message)
}

/// 重命名会话。
///
/// # Errors
///
/// 标题为空或会话不存在时返回错误。
#[tauri::command]
pub fn conversation_rename(
    state: State<'_, AppState>,
    id: String,
    title: String,
) -> Result<Conversation, String> {
    state
        .db
        .with_connection(|connection| repo::rename_conversation(connection, &id, &title))
        .map_err(to_message)
}

/// 删除会话（消息级联删除）。
///
/// # Errors
///
/// 会话不存在或数据库写入失败时返回错误。
#[tauri::command]
pub fn conversation_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .db
        .with_connection(|connection| repo::delete_conversation(connection, &id))
        .map_err(to_message)
}
