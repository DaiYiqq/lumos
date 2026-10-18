//! 摄取相关命令：导入、队列控制、文档列表。

use std::collections::HashSet;
use std::path::PathBuf;

use tauri::State;
use walkdir::WalkDir;

use crate::db::models::{DocumentSummary, IngestJob};
use crate::db::repo::{documents, jobs, libraries};
use crate::error::AppError;
use crate::ingest::extract;
use crate::state::AppState;

/// 把领域错误转换成前端可直接展示的一句话。
fn to_message(error: AppError) -> String {
    error.user_message()
}

/// 导入文件或文件夹：展开目录、过滤支持的格式、去重后创建任务。
///
/// # Errors
///
/// 知识库不存在、路径不可读或任务写入失败时返回面向用户的错误。
#[tauri::command]
pub fn ingest_import_paths(
    state: State<'_, AppState>,
    library_id: String,
    paths: Vec<String>,
) -> Result<usize, String> {
    let library = state
        .db
        .with_connection(|connection| libraries::get_library(connection, &library_id))
        .map_err(to_message)?;
    if library.is_none() {
        return Err("找不到对应的知识库，可能已被删除".to_string());
    }

    let queued: HashSet<String> = state
        .db
        .with_connection(|connection| jobs::list_jobs(connection, &library_id))
        .map_err(to_message)?
        .into_iter()
        .filter(|job| job.status == "pending" || job.status == "running")
        .map(|job| job.path)
        .collect();

    let mut files: Vec<PathBuf> = Vec::new();
    for raw in paths {
        let path = PathBuf::from(raw);
        if path.is_dir() {
            for entry in WalkDir::new(&path)
                .follow_links(false)
                .into_iter()
                .flatten()
            {
                if entry.file_type().is_file() && extract::supports(entry.path()) {
                    files.push(entry.path().to_path_buf());
                }
            }
        } else if path.is_file() && extract::supports(&path) {
            files.push(path);
        }
    }
    files.sort();
    files.dedup();

    let mut created = 0usize;
    for file in files {
        let path_text = file.to_string_lossy().to_string();
        if queued.contains(&path_text) {
            continue;
        }
        let job = state
            .db
            .with_connection(|connection| jobs::create_job(connection, &library_id, &path_text))
            .map_err(to_message)?;
        state.queue.enqueue(job.id);
        created += 1;
    }

    if created == 0 {
        return Err("没有新的可导入文件（可能已导入过，或格式不在支持范围内）".to_string());
    }
    Ok(created)
}

/// 列出知识库下的文档。
///
/// # Errors
///
/// 数据库查询失败时返回面向用户的错误。
#[tauri::command]
pub fn document_list(
    state: State<'_, AppState>,
    library_id: String,
) -> Result<Vec<DocumentSummary>, String> {
    state
        .db
        .with_connection(|connection| documents::list_documents(connection, &library_id))
        .map_err(to_message)
}

/// 删除单个文档（含分块）。
///
/// # Errors
///
/// 文档不存在或数据库写入失败时返回面向用户的错误。
#[tauri::command]
pub fn document_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .db
        .with_connection(|connection| documents::delete_document(connection, &id))
        .map_err(to_message)
}

/// 列出知识库的导入任务。
///
/// # Errors
///
/// 数据库查询失败时返回面向用户的错误。
#[tauri::command]
pub fn ingest_job_list(
    state: State<'_, AppState>,
    library_id: String,
) -> Result<Vec<IngestJob>, String> {
    state
        .db
        .with_connection(|connection| jobs::list_jobs(connection, &library_id))
        .map_err(to_message)
}

/// 暂停导入队列（当前文件处理完后暂停）。
#[tauri::command]
pub fn ingest_pause(state: State<'_, AppState>) -> bool {
    state.queue.pause();
    state.queue.is_paused()
}

/// 恢复导入队列。
#[tauri::command]
pub fn ingest_resume(state: State<'_, AppState>) -> bool {
    state.queue.resume();
    state.queue.is_paused()
}

/// 查询队列是否暂停。
#[tauri::command]
pub fn ingest_queue_state(state: State<'_, AppState>) -> bool {
    state.queue.is_paused()
}

/// 取消一个导入任务。
///
/// # Errors
///
/// 数据库写入失败时返回面向用户的错误。
#[tauri::command]
pub fn ingest_cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    state.queue.cancel(&job_id);
    state
        .db
        .with_connection(|connection| {
            jobs::update_job(connection, &job_id, "cancelled", 0, Some("任务已取消"))
        })
        .map_err(to_message)
}
