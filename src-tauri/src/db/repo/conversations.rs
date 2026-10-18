//! 会话与消息仓储：问答历史的持久化。

use rusqlite::{Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use crate::db::models::{Conversation, ConversationSummary, StoredMessage};
use crate::error::{AppError, AppResult};

/// 创建会话。
///
/// # Errors
///
/// 数据库写入失败时返回错误。
pub fn create_conversation(
    connection: &Connection,
    library_id: &str,
    title: &str,
) -> AppResult<Conversation> {
    let id = Uuid::new_v4().to_string();
    let title = title.trim();
    let title = if title.is_empty() { "新对话" } else { title };
    connection.execute(
        "INSERT INTO conversations (id, library_id, title, created_at, updated_at) \
         VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        params![id, library_id, title],
    )?;
    get_conversation(connection, &id)?.ok_or(AppError::Internal("会话创建失败".to_string()))
}

/// 按 ID 获取会话。
///
/// # Errors
///
/// 数据库查询失败时返回错误；不存在返回 `Ok(None)`。
pub fn get_conversation(connection: &Connection, id: &str) -> AppResult<Option<Conversation>> {
    let conversation = connection
        .query_row(
            "SELECT id, library_id, title, created_at, updated_at FROM conversations WHERE id = ?1",
            [id],
            map_conversation,
        )
        .optional()?;
    Ok(conversation)
}

/// 列出某知识库的会话，最新在前。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
pub fn list_conversations(
    connection: &Connection,
    library_id: &str,
) -> AppResult<Vec<ConversationSummary>> {
    let mut statement = connection.prepare(
        "SELECT c.id, c.library_id, c.title, c.created_at, c.updated_at, \
                (SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id) AS message_count \
         FROM conversations c WHERE c.library_id = ?1 ORDER BY c.updated_at DESC",
    )?;
    let rows = statement.query_map([library_id], |row| {
        Ok(ConversationSummary {
            conversation: map_conversation(row)?,
            message_count: row.get(5)?,
        })
    })?;
    let mut conversations = Vec::new();
    for row in rows {
        conversations.push(row?);
    }
    Ok(conversations)
}

/// 重命名会话。
///
/// # Errors
///
/// 标题为空、会话不存在或数据库写入失败时返回错误。
pub fn rename_conversation(
    connection: &Connection,
    id: &str,
    title: &str,
) -> AppResult<Conversation> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("会话标题不能为空".to_string()));
    }
    let changed = connection.execute(
        "UPDATE conversations SET title = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ?2",
        params![trimmed, id],
    )?;
    if changed == 0 {
        return Err(AppError::InvalidInput("会话不存在或已被删除".to_string()));
    }
    get_conversation(connection, id)?
        .ok_or_else(|| AppError::InvalidInput("会话不存在".to_string()))
}

/// 删除会话（消息级联删除）。
///
/// # Errors
///
/// 会话不存在或数据库写入失败时返回错误。
pub fn delete_conversation(connection: &Connection, id: &str) -> AppResult<()> {
    let changed = connection.execute("DELETE FROM conversations WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(AppError::InvalidInput("会话不存在或已被删除".to_string()));
    }
    Ok(())
}

/// 读取会话内的全部消息（按时间顺序）。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
pub fn list_messages(
    connection: &Connection,
    conversation_id: &str,
) -> AppResult<Vec<StoredMessage>> {
    let mut statement = connection.prepare(
        "SELECT id, conversation_id, role, content, citations_json, created_at \
         FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC, rowid ASC",
    )?;
    let rows = statement.query_map([conversation_id], map_message)?;
    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    Ok(messages)
}

/// 追加一条消息，并刷新会话的 `updated_at`。
///
/// # Errors
///
/// 角色非法、会话不存在或数据库写入失败时返回错误。
pub fn append_message(
    connection: &Connection,
    conversation_id: &str,
    role: &str,
    content: &str,
    citations_json: Option<&str>,
) -> AppResult<StoredMessage> {
    if !matches!(role, "user" | "assistant" | "system") {
        return Err(AppError::InvalidInput("未知的消息角色".to_string()));
    }

    let id = Uuid::new_v4().to_string();
    let transaction = connection.unchecked_transaction()?;
    transaction.execute(
        "INSERT INTO messages (id, conversation_id, role, content, citations_json, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        params![id, conversation_id, role, content, citations_json],
    )?;
    transaction.execute(
        "UPDATE conversations SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
        [conversation_id],
    )?;
    transaction.commit()?;

    let message = connection
        .query_row(
            "SELECT id, conversation_id, role, content, citations_json, created_at \
             FROM messages WHERE id = ?1",
            [&id],
            map_message,
        )
        .optional()?
        .ok_or_else(|| AppError::Internal("消息写入失败".to_string()))?;
    Ok(message)
}

/// 会话标题：截断长问题。
#[must_use]
pub fn title_from_question(question: &str) -> String {
    let cleaned = question.trim().replace('\n', " ");
    let mut title: String = cleaned.chars().take(24).collect();
    if cleaned.chars().count() > 24 {
        title.push('…');
    }
    if title.is_empty() {
        "新对话".to_string()
    } else {
        title
    }
}

/// 映射会话行。
fn map_conversation(row: &Row<'_>) -> rusqlite::Result<Conversation> {
    Ok(Conversation {
        id: row.get(0)?,
        library_id: row.get(1)?,
        title: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

/// 映射消息行。
fn map_message(row: &Row<'_>) -> rusqlite::Result<StoredMessage> {
    Ok(StoredMessage {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        citations_json: row.get(4)?,
        created_at: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::db::repo::libraries::create_library;

    #[test]
    fn conversation_lifecycle_and_messages() {
        let database = Database::open_in_memory().expect("内存库应可创建");
        database.migrate().expect("迁移应成功");
        let library = database
            .with_connection(|connection| create_library(connection, "问答"))
            .expect("建库应成功");

        let conversation = database
            .with_connection(|connection| {
                create_conversation(
                    connection,
                    &library.id,
                    &title_from_question("合同违约金上限是怎么约定的"),
                )
            })
            .expect("建会话应成功");
        assert!(conversation.title.starts_with("合同违约金上限"));

        database
            .with_connection(|connection| {
                append_message(connection, &conversation.id, "user", "问题", None)?;
                append_message(
                    connection,
                    &conversation.id,
                    "assistant",
                    "回答 [1]",
                    Some("[{\"index\":1}]"),
                )
            })
            .expect("写消息应成功");

        let messages = database
            .with_connection(|connection| list_messages(connection, &conversation.id))
            .expect("读消息应成功");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].role, "assistant");
        assert!(messages[1].citations_json.is_some());

        let summaries = database
            .with_connection(|connection| list_conversations(connection, &library.id))
            .expect("列表应成功");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].message_count, 2);

        database
            .with_connection(|connection| {
                rename_conversation(connection, &conversation.id, "新标题")
            })
            .expect("重命名应成功");
        database
            .with_connection(|connection| delete_conversation(connection, &conversation.id))
            .expect("删除应成功");

        let remaining: i64 = database
            .with_connection(|connection| {
                let count: i64 =
                    connection.query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))?;
                Ok(count)
            })
            .expect("统计应成功");
        assert_eq!(remaining, 0);
    }
}
