//! 分块与向量仓储：向量写入、KNN 检索、BM25 检索。

use std::collections::HashMap;

use rusqlite::{Connection, params};
use serde::Serialize;

use crate::error::{AppError, AppResult};

/// 存储中的分块（含正文与位置）。
#[derive(Debug, Clone)]
pub struct StoredChunk {
    /// chunks.id。
    pub id: i64,
    /// 所属文档。
    pub document_id: String,
    /// 分块正文。
    pub content: String,
    /// 页码。
    pub page_no: Option<i64>,
    /// 章节路径。
    pub section_path: Option<String>,
}

/// 向量检索命中。
#[derive(Debug, Clone, Copy)]
pub struct VectorHit {
    /// 分块 ID。
    pub chunk_id: i64,
    /// 距离（越小越相似）。
    pub distance: f64,
}

/// BM25 检索命中。
#[derive(Debug, Clone, Copy)]
pub struct KeywordHit {
    /// 分块 ID。
    pub chunk_id: i64,
    /// FTS5 的 bm25 分数（越小越相关）。
    pub score: f64,
}

/// 用于组装检索结果的完整分块信息。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkDetail {
    /// 分块 ID。
    pub chunk_id: i64,
    /// 文档 ID。
    pub document_id: String,
    /// 文档标题。
    pub document_title: String,
    /// 文档路径。
    pub document_path: String,
    /// 页码。
    pub page_no: Option<i64>,
    /// 章节路径。
    pub section_path: Option<String>,
    /// 分块正文。
    pub content: String,
    /// 估算 token 数。
    pub token_count: i64,
}

/// 列出某文档的全部分块（按顺序）。
///
/// # Errors
///
/// 查询失败时返回错误。
pub fn list_chunks_for_document(
    connection: &Connection,
    document_id: &str,
) -> AppResult<Vec<StoredChunk>> {
    let mut statement = connection.prepare(
        "SELECT id, document_id, content, page_no, section_path \
         FROM chunks WHERE document_id = ?1 ORDER BY ordinal ASC",
    )?;
    let rows = statement.query_map([document_id], map_stored_chunk)?;
    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row?);
    }
    Ok(chunks)
}

/// 找出尚未写入向量的分块（用于模型就绪后的补写）。
///
/// # Errors
///
/// 查询失败时返回错误。
pub fn list_chunks_missing_vectors(
    connection: &Connection,
    limit: usize,
) -> AppResult<Vec<StoredChunk>> {
    let mut statement = connection.prepare(
        "SELECT c.id, c.document_id, c.content, c.page_no, c.section_path \
         FROM chunks c \
         WHERE NOT EXISTS (SELECT 1 FROM chunks_vec v WHERE v.chunk_id = c.id) \
         ORDER BY c.id ASC LIMIT ?1",
    )?;
    let rows = statement.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], map_stored_chunk)?;
    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row?);
    }
    Ok(chunks)
}

/// 批量写入向量（同事务）。
///
/// # Errors
///
/// 向量维度不是 512 或写入失败时返回错误。
pub fn insert_vectors(connection: &Connection, vectors: &[(i64, Vec<f32>)]) -> AppResult<()> {
    if vectors.is_empty() {
        return Ok(());
    }

    let transaction = connection.unchecked_transaction()?;
    {
        let mut statement = transaction
            .prepare("INSERT OR REPLACE INTO chunks_vec (chunk_id, embedding) VALUES (?1, ?2)")?;
        for (chunk_id, vector) in vectors {
            if vector.len() != crate::embedding::EMBEDDING_DIM {
                return Err(AppError::Internal(format!(
                    "嵌入维度不匹配：期望 {}，实际 {}",
                    crate::embedding::EMBEDDING_DIM,
                    vector.len()
                )));
            }
            statement.execute(params![chunk_id, vector_to_blob(vector)])?;
        }
    }
    transaction.commit()?;
    Ok(())
}

/// 向量 KNN 检索。
///
/// `k` 会扩大后再按知识库过滤，避免先取 k 条再过滤导致结果不足。
///
/// # Errors
///
/// 查询失败时返回错误。
pub fn vector_search(
    connection: &Connection,
    query: &[f32],
    library_id: &str,
    limit: usize,
) -> AppResult<Vec<VectorHit>> {
    if query.len() != crate::embedding::EMBEDDING_DIM {
        return Err(AppError::Internal("查询向量维度不正确".to_string()));
    }

    let fetch_k = i64::try_from((limit.max(1)) * 5).unwrap_or(i64::MAX);
    let mut statement = connection.prepare(
        "SELECT v.chunk_id, v.distance \
         FROM chunks_vec v \
         JOIN chunks c ON c.id = v.chunk_id \
         JOIN documents d ON d.id = c.document_id \
         WHERE v.embedding MATCH ?1 AND k = ?2 AND d.library_id = ?3 \
         ORDER BY v.distance ASC LIMIT ?4",
    )?;
    let rows = statement.query_map(
        params![
            vector_to_blob(query),
            fetch_k,
            library_id,
            i64::try_from(limit.max(1)).unwrap_or(i64::MAX)
        ],
        |row| {
            Ok(VectorHit {
                chunk_id: row.get(0)?,
                distance: row.get(1)?,
            })
        },
    )?;

    let mut hits = Vec::new();
    for row in rows {
        hits.push(row?);
    }
    Ok(hits)
}

/// BM25 关键字检索（FTS5 trigram）。
///
/// # Errors
///
/// 查询失败时返回错误；空查询返回空结果。
pub fn keyword_search(
    connection: &Connection,
    query: &str,
    library_id: &str,
    limit: usize,
) -> AppResult<Vec<KeywordHit>> {
    let match_query = build_match_query(query);
    if match_query.is_empty() {
        return Ok(Vec::new());
    }

    let mut statement = connection.prepare(
        "SELECT c.id, bm25(chunks_fts) AS score \
         FROM chunks_fts \
         JOIN chunks c ON c.id = chunks_fts.rowid \
         JOIN documents d ON d.id = c.document_id \
         WHERE chunks_fts MATCH ?1 AND d.library_id = ?2 \
         ORDER BY score ASC LIMIT ?3",
    )?;
    let rows = statement.query_map(
        params![
            match_query,
            library_id,
            i64::try_from(limit.max(1)).unwrap_or(i64::MAX)
        ],
        |row| {
            Ok(KeywordHit {
                chunk_id: row.get(0)?,
                score: row.get(1)?,
            })
        },
    )?;

    let mut hits = Vec::new();
    for row in rows {
        hits.push(row?);
    }
    Ok(hits)
}

/// 按给定顺序读取分块详情。
///
/// # Errors
///
/// 查询失败时返回错误。
pub fn chunk_details(connection: &Connection, chunk_ids: &[i64]) -> AppResult<Vec<ChunkDetail>> {
    if chunk_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut statement = connection.prepare(
        "SELECT c.id, c.document_id, d.title, d.path, c.page_no, c.section_path, c.content, c.token_count \
         FROM chunks c JOIN documents d ON d.id = c.document_id \
         WHERE c.id = ?1",
    )?;

    let mut by_id: HashMap<i64, ChunkDetail> = HashMap::new();
    for chunk_id in chunk_ids {
        if let Ok(detail) = statement.query_row([chunk_id], |row| {
            Ok(ChunkDetail {
                chunk_id: row.get(0)?,
                document_id: row.get(1)?,
                document_title: row.get(2)?,
                document_path: row.get(3)?,
                page_no: row.get(4)?,
                section_path: row.get(5)?,
                content: row.get(6)?,
                token_count: row.get(7)?,
            })
        }) {
            by_id.insert(*chunk_id, detail);
        }
    }

    Ok(chunk_ids
        .iter()
        .filter_map(|chunk_id| by_id.get(chunk_id).cloned())
        .collect())
}

/// 把 f32 向量转成 sqlite-vec 期望的小端 blob。
fn vector_to_blob(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for value in vector {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// 构造 FTS5 查询。
///
/// 对中文查询按 3 字滑窗拆成多个词（trigram 索引只索引长度 ≥ 3 的子串），
/// 这样「违约金最多能赔多少」也能命中包含「违约金」的段落；
/// 拉丁字母 / 编号类词保持原样并用引号包裹，避免被当成 FTS 语法。
fn build_match_query(query: &str) -> String {
    let mut terms: Vec<String> = Vec::new();

    for token in query.split_whitespace() {
        let characters: Vec<char> = token.chars().collect();
        let has_cjk = characters.iter().any(|character| is_cjk(*character));

        if has_cjk && characters.len() >= 3 {
            for window in characters.windows(3) {
                terms.push(quote_term(&window.iter().collect::<String>()));
            }
        } else if !token.is_empty() {
            terms.push(quote_term(token));
        }
    }

    terms.sort();
    terms.dedup();
    terms.join(" OR ")
}

/// 给单个词加引号并转义内部引号。
fn quote_term(term: &str) -> String {
    format!("\"{}\"", term.replace('"', "\"\""))
}

/// 判断是否为 CJK 字符。
fn is_cjk(character: char) -> bool {
    matches!(character as u32,
        0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF)
}

/// 把查询行映射为 [`StoredChunk`]。
fn map_stored_chunk(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredChunk> {
    Ok(StoredChunk {
        id: row.get(0)?,
        document_id: row.get(1)?,
        content: row.get(2)?,
        page_no: row.get(3)?,
        section_path: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup() -> Database {
        let database = Database::open_in_memory().expect("内存库应可创建");
        database.migrate().expect("迁移应成功");
        database
    }

    fn insert_doc(connection: &Connection, document_id: &str, chunk_text: &str) -> i64 {
        connection
            .execute(
                "INSERT INTO documents (id, library_id, path, title, created_at, updated_at) \
                 VALUES (?1, (SELECT id FROM libraries LIMIT 1), '/tmp/x.md', 'X', \
                         '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z')",
                [document_id],
            )
            .expect("文档应可写入");
        connection
            .execute(
                "INSERT INTO chunks (document_id, ordinal, content, created_at) \
                 VALUES (?1, 0, ?2, '2026-01-01T00:00:00.000Z')",
                rusqlite::params![document_id, chunk_text],
            )
            .expect("分块应可写入");
        connection.last_insert_rowid()
    }

    #[test]
    fn stores_vectors_and_searches_by_distance() {
        let database = setup();
        let library = database
            .with_connection(|connection| {
                crate::db::repo::libraries::create_library(connection, "向量")
            })
            .expect("建库应成功");

        let chunk_id = database
            .with_connection(|connection| Ok(insert_doc(connection, "doc-1", "违约金上限条款")))
            .expect("写入应成功");

        let mut vector = vec![0.0_f32; crate::embedding::EMBEDDING_DIM];
        vector[0] = 1.0;
        database
            .with_connection(|connection| insert_vectors(connection, &[(chunk_id, vector.clone())]))
            .expect("向量写入应成功");

        let hits = database
            .with_connection(|connection| vector_search(connection, &vector, &library.id, 5))
            .expect("检索应成功");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk_id, chunk_id);
        assert!(hits[0].distance.abs() < 1e-4);

        // 删除分块后向量应被触发器清理
        database
            .with_connection(|connection| {
                connection.execute("DELETE FROM chunks WHERE id = ?1", [chunk_id])?;
                Ok(())
            })
            .expect("删除应成功");
        let remaining = database
            .with_connection(|connection| {
                let count: i64 =
                    connection
                        .query_row("SELECT COUNT(*) FROM chunks_vec", [], |row| row.get(0))?;
                Ok(count)
            })
            .expect("统计应成功");
        assert_eq!(remaining, 0);
    }

    #[test]
    fn bm25_finds_chinese_substring() {
        let database = setup();
        let library = database
            .with_connection(|connection| {
                crate::db::repo::libraries::create_library(connection, "关键词")
            })
            .expect("建库应成功");
        database
            .with_connection(|connection| {
                Ok(insert_doc(connection, "doc-2", "对比学习用于医学图像分割"))
            })
            .expect("写入应成功");

        let hits = database
            .with_connection(|connection| keyword_search(connection, "医学图像", &library.id, 5))
            .expect("检索应成功");
        assert_eq!(hits.len(), 1);

        let none = database
            .with_connection(|connection| {
                keyword_search(connection, "完全无关的词", &library.id, 5)
            })
            .expect("检索应成功");
        assert!(none.is_empty());
    }
}
