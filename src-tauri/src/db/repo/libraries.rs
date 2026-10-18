//! 知识库仓储：创建、列表、重命名、删除。
//!
//! 所有函数都只接收 `&Connection`，不感知 Tauri，方便单元测试。

use rusqlite::{Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use crate::db::models::{Library, LibrarySummary};
use crate::error::{AppError, AppResult};

/// 知识库名称最大字符数（按 Unicode 字符计，不是字节）。
const MAX_NAME_LENGTH: usize = 80;

/// 时间戳表达式：RFC3339 UTC，毫秒精度。
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";

/// 创建知识库。
///
/// # Errors
///
/// 名称为空、超长或重名时返回 [`AppError`]；数据库写入失败时同样返回错误。
pub fn create_library(connection: &Connection, name: &str) -> AppResult<Library> {
    let name = normalize_name(name)?;
    if name_exists(connection, &name, None)? {
        return Err(AppError::DuplicateLibraryName);
    }

    let id = Uuid::new_v4().to_string();
    connection.execute(
        &format!(
            "INSERT INTO libraries (id, name, created_at, updated_at) \
             VALUES (?1, ?2, {NOW}, {NOW})"
        ),
        params![id, name],
    )?;

    get_library(connection, &id)?.ok_or(AppError::LibraryNotFound)
}

/// 按 ID 获取知识库。
///
/// # Errors
///
/// 数据库查询失败时返回错误；不存在时返回 `Ok(None)`。
pub fn get_library(connection: &Connection, id: &str) -> AppResult<Option<Library>> {
    let library = connection
        .query_row(
            "SELECT id, name, created_at, updated_at, embedding_model, chunk_config \
             FROM libraries WHERE id = ?1",
            [id],
            map_library,
        )
        .optional()?;
    Ok(library)
}

/// 列出全部知识库及其统计信息，按最后更新时间倒序。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
pub fn list_libraries(connection: &Connection) -> AppResult<Vec<LibrarySummary>> {
    let mut statement = connection.prepare(
        "SELECT l.id, l.name, l.created_at, l.updated_at, l.embedding_model, l.chunk_config, \
                (SELECT COUNT(*) FROM documents d WHERE d.library_id = l.id) AS document_count, \
                (SELECT COUNT(*) FROM chunks c \
                   JOIN documents d2 ON c.document_id = d2.id \
                  WHERE d2.library_id = l.id) AS chunk_count, \
                COALESCE((SELECT SUM(d3.size) FROM documents d3 WHERE d3.library_id = l.id), 0) AS size_bytes \
         FROM libraries l \
         ORDER BY l.updated_at DESC, l.name COLLATE NOCASE ASC",
    )?;

    let rows = statement.query_map([], map_summary)?;
    let mut libraries = Vec::new();
    for row in rows {
        libraries.push(row?);
    }
    Ok(libraries)
}

/// 重命名知识库，并刷新 `updated_at`。
///
/// # Errors
///
/// 名称为空、超长、与其他库重名或目标库不存在时返回 [`AppError`]。
pub fn rename_library(connection: &Connection, id: &str, name: &str) -> AppResult<Library> {
    let name = normalize_name(name)?;
    if name_exists(connection, &name, Some(id))? {
        return Err(AppError::DuplicateLibraryName);
    }

    let changed = connection.execute(
        &format!("UPDATE libraries SET name = ?1, updated_at = {NOW} WHERE id = ?2"),
        params![name, id],
    )?;
    if changed == 0 {
        return Err(AppError::LibraryNotFound);
    }

    get_library(connection, id)?.ok_or(AppError::LibraryNotFound)
}

/// 删除知识库；`documents` / `chunks` / `conversations` / `messages` 由外键级联清理。
///
/// # Errors
///
/// 目标库不存在时返回 [`AppError::LibraryNotFound`]。
pub fn delete_library(connection: &Connection, id: &str) -> AppResult<()> {
    let changed = connection.execute("DELETE FROM libraries WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(AppError::LibraryNotFound);
    }
    Ok(())
}

/// 校验并整理名称：去除首尾空白，拒绝空串与超长名称。
fn normalize_name(name: &str) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("知识库名称不能为空".to_string()));
    }
    if trimmed.chars().count() > MAX_NAME_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "知识库名称不能超过 {MAX_NAME_LENGTH} 个字符"
        )));
    }
    Ok(trimmed.to_string())
}

/// 判断名称是否已被占用（忽略大小写），`exclude_id` 用于重命名时排除自己。
fn name_exists(connection: &Connection, name: &str, exclude_id: Option<&str>) -> AppResult<bool> {
    let exists = match exclude_id {
        Some(id) => connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM libraries WHERE name = ?1 COLLATE NOCASE AND id <> ?2)",
            params![name, id],
            |row| row.get::<_, bool>(0),
        )?,
        None => connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM libraries WHERE name = ?1 COLLATE NOCASE)",
            [name],
            |row| row.get::<_, bool>(0),
        )?,
    };
    Ok(exists)
}

/// 把查询行映射为 [`Library`]。
fn map_library(row: &Row<'_>) -> rusqlite::Result<Library> {
    Ok(Library {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        embedding_model: row.get(4)?,
        chunk_config: row.get(5)?,
    })
}

/// 把查询行映射为 [`LibrarySummary`]。
fn map_summary(row: &Row<'_>) -> rusqlite::Result<LibrarySummary> {
    let library = map_library(row)?;
    Ok(LibrarySummary {
        library,
        document_count: row.get(6)?,
        chunk_count: row.get(7)?,
        size_bytes: row.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    /// 创建并迁移一个内存数据库。
    fn setup() -> Database {
        let database = Database::open_in_memory().expect("内存库应可创建");
        database.migrate().expect("迁移应成功");
        database
    }

    #[test]
    fn creates_lists_and_reads_library() {
        let database = setup();
        let created = database
            .with_connection(|connection| create_library(connection, "  论文库  "))
            .expect("创建应成功");
        assert_eq!(created.name, "论文库");
        assert!(!created.id.is_empty());

        let summaries = database
            .with_connection(list_libraries)
            .expect("列表应成功");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].library.name, "论文库");
        assert_eq!(summaries[0].document_count, 0);
        assert_eq!(summaries[0].chunk_count, 0);
        assert_eq!(summaries[0].size_bytes, 0);

        let fetched = database
            .with_connection(|connection| get_library(connection, &created.id))
            .expect("读取应成功")
            .expect("知识库应存在");
        assert_eq!(fetched.id, created.id);
    }

    #[test]
    fn rejects_blank_and_duplicate_names() {
        let database = setup();
        database
            .with_connection(|connection| create_library(connection, "合同库"))
            .expect("首次创建应成功");

        let blank = database.with_connection(|connection| create_library(connection, "   "));
        assert!(matches!(blank, Err(AppError::InvalidInput(_))));

        let too_long = "x".repeat(MAX_NAME_LENGTH + 1);
        let long_error =
            database.with_connection(|connection| create_library(connection, &too_long));
        assert!(matches!(long_error, Err(AppError::InvalidInput(_))));

        let duplicate = database.with_connection(|connection| create_library(connection, "合同库"));
        assert!(matches!(duplicate, Err(AppError::DuplicateLibraryName)));

        let duplicate_case =
            database.with_connection(|connection| create_library(connection, "合同库"));
        assert!(matches!(
            duplicate_case,
            Err(AppError::DuplicateLibraryName)
        ));
    }

    #[test]
    fn renames_library_and_rejects_conflicts() {
        let database = setup();
        let first = database
            .with_connection(|connection| create_library(connection, "笔记"))
            .expect("创建应成功");
        let second = database
            .with_connection(|connection| create_library(connection, "日记"))
            .expect("创建应成功");

        let renamed = database
            .with_connection(|connection| rename_library(connection, &first.id, "三年日记"))
            .expect("重命名应成功");
        assert_eq!(renamed.name, "三年日记");

        let conflict = database
            .with_connection(|connection| rename_library(connection, &first.id, &second.name));
        assert!(matches!(conflict, Err(AppError::DuplicateLibraryName)));

        let missing =
            database.with_connection(|connection| rename_library(connection, "not-exist", "随便"));
        assert!(matches!(missing, Err(AppError::LibraryNotFound)));
    }

    #[test]
    fn delete_removes_library_and_cascades() {
        let database = setup();
        let library = database
            .with_connection(|connection| create_library(connection, "待删除"))
            .expect("创建应成功");

        database
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO documents (id, library_id, path, title, size, created_at, updated_at) \
                     VALUES ('doc-1', ?1, '/tmp/a.md', 'A', 128, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z')",
                    [&library.id],
                )?;
                connection.execute(
                    "INSERT INTO chunks (document_id, ordinal, content, created_at) \
                     VALUES ('doc-1', 0, '违约金上限为合同总额的百分之二十', '2026-01-01T00:00:00.000Z')",
                    [],
                )?;
                Ok(())
            })
            .expect("写入测试数据应成功");

        database
            .with_connection(|connection| delete_library(connection, &library.id))
            .expect("删除应成功");

        let (libraries, documents, chunks) = database
            .with_connection(|connection| {
                let libraries: i64 =
                    connection.query_row("SELECT COUNT(*) FROM libraries", [], |row| row.get(0))?;
                let documents: i64 =
                    connection.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
                let chunks: i64 =
                    connection.query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;
                Ok((libraries, documents, chunks))
            })
            .expect("统计应成功");

        assert_eq!((libraries, documents, chunks), (0, 0, 0));

        let again = database.with_connection(|connection| delete_library(connection, &library.id));
        assert!(matches!(again, Err(AppError::LibraryNotFound)));
    }

    #[test]
    fn fts_trigger_keeps_index_in_sync() {
        let database = setup();
        let library = database
            .with_connection(|connection| create_library(connection, "检索"))
            .expect("创建应成功");

        database
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO documents (id, library_id, path, title, created_at, updated_at) \
                     VALUES ('doc-2', ?1, '/tmp/b.md', 'B', '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z')",
                    [&library.id],
                )?;
                connection.execute(
                    "INSERT INTO chunks (document_id, ordinal, content, created_at) \
                     VALUES ('doc-2', 0, '对比学习在医学图像分割中的应用', '2026-01-01T00:00:00.000Z')",
                    [],
                )?;
                Ok(())
            })
            .expect("写入测试数据应成功");

        database
            .with_connection(|connection| {
                let matches: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM chunks_fts WHERE chunks_fts MATCH '医学图像'",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!(matches, 1, "trigram 索引应能命中中文子串");
                Ok(())
            })
            .expect("FTS 查询应成功");
    }
}
