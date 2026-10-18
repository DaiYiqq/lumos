//! 摄取任务仓储。

use rusqlite::{Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use crate::db::models::IngestJob;
use crate::error::{AppError, AppResult};

/// 创建一个待处理任务。
///
/// # Errors
///
/// 数据库写入失败时返回错误。
pub fn create_job(connection: &Connection, library_id: &str, path: &str) -> AppResult<IngestJob> {
    let id = Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO ingest_jobs (id, library_id, path, status, created_at, updated_at) \
         VALUES (?1, ?2, ?3, 'pending', \
                 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        params![id, library_id, path],
    )?;
    get_job(connection, &id)?.ok_or(AppError::LibraryNotFound)
}

/// 按 ID 读取任务。
///
/// # Errors
///
/// 数据库查询失败时返回错误；不存在返回 `Ok(None)`。
pub fn get_job(connection: &Connection, id: &str) -> AppResult<Option<IngestJob>> {
    let job = connection
        .query_row(
            "SELECT id, library_id, path, status, chunk_count, message, created_at, updated_at \
             FROM ingest_jobs WHERE id = ?1",
            [id],
            map_job,
        )
        .optional()?;
    Ok(job)
}

/// 更新任务状态与结果。
///
/// # Errors
///
/// 数据库写入失败时返回错误。
pub fn update_job(
    connection: &Connection,
    id: &str,
    status: &str,
    chunk_count: i64,
    message: Option<&str>,
) -> AppResult<()> {
    connection.execute(
        "UPDATE ingest_jobs SET status = ?1, chunk_count = ?2, message = ?3, \
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?4",
        params![status, chunk_count, message, id],
    )?;
    Ok(())
}

/// 列出某知识库的任务，最新在前。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
pub fn list_jobs(connection: &Connection, library_id: &str) -> AppResult<Vec<IngestJob>> {
    let mut statement = connection.prepare(
        "SELECT id, library_id, path, status, chunk_count, message, created_at, updated_at \
         FROM ingest_jobs WHERE library_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = statement.query_map([library_id], map_job)?;
    let mut jobs = Vec::new();
    for row in rows {
        jobs.push(row?);
    }
    Ok(jobs)
}

/// 读取所有未完成任务（pending / running），用于应用启动后续跑。
///
/// # Errors
///
/// 数据库查询失败时返回错误。
pub fn list_unfinished_jobs(connection: &Connection) -> AppResult<Vec<IngestJob>> {
    let mut statement = connection.prepare(
        "SELECT id, library_id, path, status, chunk_count, message, created_at, updated_at \
         FROM ingest_jobs WHERE status IN ('pending', 'running') ORDER BY created_at ASC",
    )?;
    let rows = statement.query_map([], map_job)?;
    let mut jobs = Vec::new();
    for row in rows {
        jobs.push(row?);
    }
    Ok(jobs)
}

/// 把因进程退出而卡在 running 的任务重置为 pending。
///
/// # Errors
///
/// 数据库写入失败时返回错误。
pub fn reset_running_jobs(connection: &Connection) -> AppResult<usize> {
    let changed = connection.execute(
        "UPDATE ingest_jobs SET status = 'pending', \
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE status = 'running'",
        [],
    )?;
    Ok(changed)
}

/// 把查询行映射为 [`IngestJob`]。
fn map_job(row: &Row<'_>) -> rusqlite::Result<IngestJob> {
    Ok(IngestJob {
        id: row.get(0)?,
        library_id: row.get(1)?,
        path: row.get(2)?,
        status: row.get(3)?,
        chunk_count: row.get(4)?,
        message: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::db::repo::libraries::create_library;

    #[test]
    fn jobs_survive_restart_and_reset_running_status() {
        let database = Database::open_in_memory().expect("内存库应可创建");
        database.migrate().expect("迁移应成功");
        let library = database
            .with_connection(|connection| create_library(connection, "重启测试"))
            .expect("建库应成功");

        let job = database
            .with_connection(|connection| create_job(connection, &library.id, "/tmp/a.md"))
            .expect("创建任务应成功");
        assert_eq!(job.status, "pending");

        database
            .with_connection(|connection| update_job(connection, &job.id, "running", 0, None))
            .expect("更新状态应成功");

        // 模拟进程退出：running 任务在启动时被重置并重新入队
        let unfinished = database
            .with_connection(|connection| {
                let reset = reset_running_jobs(connection)?;
                let jobs = list_unfinished_jobs(connection)?;
                Ok((reset, jobs))
            })
            .expect("重置应成功");

        assert_eq!(unfinished.0, 1);
        assert_eq!(unfinished.1.len(), 1);
        assert_eq!(unfinished.1[0].status, "pending");

        database
            .with_connection(|connection| update_job(connection, &job.id, "done", 12, Some("完成")))
            .expect("完成状态应成功");
        let finished = database
            .with_connection(|connection| list_jobs(connection, &library.id))
            .expect("列表应成功");
        assert_eq!(finished[0].status, "done");
        assert_eq!(finished[0].chunk_count, 12);
        assert_eq!(finished[0].message.as_deref(), Some("完成"));
    }
}
