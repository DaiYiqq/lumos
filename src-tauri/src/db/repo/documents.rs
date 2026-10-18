//! 文档仓储：写入（含分块）、列表、删除。

use rusqlite::{Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use crate::db::models::{DocumentStatus, DocumentSummary};
use crate::error::{AppError, AppResult};

/// 待写入的分块。
pub struct NewChunk {
    /// 分块正文。
    pub content: String,
    /// 所在页码（PDF 等有页概念的格式）。
    pub page_no: Option<i64>,
    /// 标题层级路径，例如 `第三章 / 3.2 违约金`。
    pub section_path: Option<String>,
    /// 在原文中的起始字符偏移。
    pub char_start: i64,
    /// 在原文中的结束字符偏移。
    pub char_end: i64,
    /// 估算 token 数。
    pub token_count: i64,
}

/// 一份待写入的文档。
pub struct NewDocument<'a> {
    /// 所属知识库。
    pub library_id: &'a str,
    /// 原始文件路径。
    pub path: &'a str,
    /// 展示标题。
    pub title: &'a str,
    /// MIME 类型。
    pub mime: &'a str,
    /// 文件大小（字节）。
    pub size: i64,
    /// 页数（可选）。
    pub page_count: Option<i64>,
    /// 内容 SHA-256（十六进制）。
    pub content_hash: &'a str,
}

/// 查询知识库内是否已有相同内容、且已就绪的文档。
///
/// 用于增量索引：重复导入同一文件时不重复解析与分块。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
pub fn find_ready_by_hash(
    connection: &Connection,
    library_id: &str,
    content_hash: &str,
) -> AppResult<Option<String>> {
    let document_id = connection
        .query_row(
            "SELECT id FROM documents \
             WHERE library_id = ?1 AND content_hash = ?2 AND status = 'ready' LIMIT 1",
            params![library_id, content_hash],
            |row| row.get(0),
        )
        .optional()?;
    Ok(document_id)
}

/// 原子写入一份文档及其全部分块。
///
/// 同一路径的旧记录会被替换（旧分块经外键级联删除），
/// 保证「一个路径只对应一份最新文档」。
///
/// # Errors
///
/// 事务执行失败时返回错误，数据库保持原状。
pub fn upsert_document(
    connection: &Connection,
    document: &NewDocument<'_>,
    chunks: &[NewChunk],
    status: DocumentStatus,
    error_msg: Option<&str>,
) -> AppResult<String> {
    let document_id = Uuid::new_v4().to_string();
    let transaction = connection.unchecked_transaction()?;

    transaction.execute(
        "DELETE FROM documents WHERE library_id = ?1 AND path = ?2",
        params![document.library_id, document.path],
    )?;

    transaction.execute(
        "INSERT INTO documents (
             id, library_id, path, title, mime, size, page_count, status, error_msg,
             created_at, updated_at, content_hash
         ) VALUES (
             ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
             strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
             strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
             ?10
         )",
        params![
            document_id,
            document.library_id,
            document.path,
            document.title,
            document.mime,
            document.size,
            document.page_count,
            status.as_str(),
            error_msg,
            document.content_hash,
        ],
    )?;

    {
        let mut statement = transaction.prepare(
            "INSERT INTO chunks (
                 document_id, ordinal, content, page_no, section_path,
                 char_start, char_end, token_count, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        )?;
        for (ordinal, chunk) in chunks.iter().enumerate() {
            statement.execute(params![
                document_id,
                i64::try_from(ordinal).unwrap_or(i64::MAX),
                chunk.content,
                chunk.page_no,
                chunk.section_path,
                chunk.char_start,
                chunk.char_end,
                chunk.token_count,
            ])?;
        }
    }

    transaction.execute(
        "UPDATE libraries SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
        [document.library_id],
    )?;

    transaction.commit()?;
    Ok(document_id)
}

/// 列出某个知识库下的全部文档。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
pub fn list_documents(
    connection: &Connection,
    library_id: &str,
) -> AppResult<Vec<DocumentSummary>> {
    let mut statement = connection.prepare(
        "SELECT d.id, d.library_id, d.path, d.title, d.mime, d.size, d.page_count,
                d.status, d.error_msg, d.created_at, d.updated_at,
                (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id) AS chunk_count
         FROM documents d
         WHERE d.library_id = ?1
         ORDER BY d.created_at DESC, d.title COLLATE NOCASE ASC",
    )?;

    let rows = statement.query_map([library_id], map_document)?;
    let mut documents = Vec::new();
    for row in rows {
        documents.push(row?);
    }
    Ok(documents)
}

/// 删除单个文档及其分块。
///
/// # Errors
///
/// 文档不存在或删除失败时返回错误。
pub fn delete_document(connection: &Connection, document_id: &str) -> AppResult<()> {
    let changed = connection.execute("DELETE FROM documents WHERE id = ?1", [document_id])?;
    if changed == 0 {
        return Err(AppError::InvalidInput("文档不存在或已被删除".to_string()));
    }
    Ok(())
}

/// 把查询行映射为 [`DocumentSummary`]。
fn map_document(row: &Row<'_>) -> rusqlite::Result<DocumentSummary> {
    Ok(DocumentSummary {
        id: row.get(0)?,
        library_id: row.get(1)?,
        path: row.get(2)?,
        title: row.get(3)?,
        mime: row.get(4)?,
        size: row.get(5)?,
        page_count: row.get(6)?,
        status: row.get(7)?,
        error_msg: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        chunk_count: row.get(11)?,
    })
}
