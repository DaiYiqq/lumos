//! 文件监听：已索引文档所在目录发生增删改时，自动入队重新索引或清理数据。
//!
//! 监听范围限定为「已有文档的父目录」，不会扫描整个磁盘；
//! 目录列表每 60 秒刷新一次，因此本次会话中新导入的文件夹也会被后续监听覆盖。

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{EventKind, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::db::Database;
use crate::db::repo::{documents, jobs};
use crate::error::{AppError, AppResult};
use crate::ingest::extract;
use crate::ingest::queue::IngestQueue;

/// 文件变化后通知前端刷新列表的事件名。
pub const CHANGED_EVENT: &str = "library://changed";

/// 变化事件载荷。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryChanged {
    library_id: String,
}

/// 文件监听器：保持存活即持续监听。
pub struct LibraryWatcher {
    _watcher: Arc<Mutex<notify::RecommendedWatcher>>,
}

/// 启动文件监听。
///
/// # Errors
///
/// 系统文件监听初始化失败时返回错误（应用仍可正常导入）。
pub fn start(
    app: AppHandle,
    database: Arc<Database>,
    queue: Arc<IngestQueue>,
) -> AppResult<LibraryWatcher> {
    let (sender, receiver) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = sender.send(event);
    })
    .map_err(|error| AppError::Internal(format!("文件监听初始化失败：{error}")))?;

    let mut watched: HashSet<PathBuf> = HashSet::new();
    for directory in watched_directories(&database)? {
        add_watch(&mut watcher, &mut watched, &directory);
    }

    let watcher = Arc::new(Mutex::new(watcher));
    let thread_watcher = Arc::clone(&watcher);
    let thread_database = Arc::clone(&database);
    let thread_queue = Arc::clone(&queue);

    std::thread::spawn(move || {
        let mut watched = watched;
        let mut pending: HashSet<PathBuf> = HashSet::new();
        let mut last_event = Instant::now();
        let mut last_scan = Instant::now();

        loop {
            match receiver.recv_timeout(Duration::from_millis(400)) {
                Ok(Ok(event)) => {
                    if matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) {
                        pending.extend(event.paths);
                    }
                    last_event = Instant::now();
                }
                Ok(Err(error)) => tracing::warn!(error = %error, "文件监听事件出错"),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if !pending.is_empty() && last_event.elapsed() >= Duration::from_millis(700) {
                let batch: Vec<PathBuf> = pending.drain().collect();
                process_paths(&app, &thread_database, &thread_queue, &batch);
            }

            if last_scan.elapsed() >= Duration::from_secs(60) {
                last_scan = Instant::now();
                refresh_watches(&thread_watcher, &thread_database, &mut watched);
            }
        }
    });

    Ok(LibraryWatcher { _watcher: watcher })
}

/// 处理一批路径事件。
fn process_paths(app: &AppHandle, database: &Database, queue: &IngestQueue, paths: &[PathBuf]) {
    for path in paths {
        if path.is_file() && extract::supports(path) {
            enqueue_changed_file(app, database, queue, path);
        } else if !path.exists() {
            remove_deleted_document(app, database, path);
        }
    }
}

/// 文件新增/修改：找到所属知识库并入队。
fn enqueue_changed_file(app: &AppHandle, database: &Database, queue: &IngestQueue, path: &Path) {
    let path_text = path.to_string_lossy().to_string();
    let library_id = match database.with_connection(|connection| {
        if let Some(library_id) = documents::find_library_by_path(connection, &path_text)? {
            return Ok(Some(library_id));
        }
        let Some(parent) = path.parent() else {
            return Ok(None);
        };
        documents::find_library_by_directory(connection, &parent.to_string_lossy())
    }) {
        Ok(Some(library_id)) => library_id,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "监听器查找知识库失败");
            return;
        }
    };

    let result = database.with_connection(|connection| {
        if jobs::has_unfinished_job(connection, &library_id, &path_text)? {
            return Ok(None);
        }
        let job = jobs::create_job(connection, &library_id, &path_text)?;
        Ok(Some(job.id))
    });

    match result {
        Ok(Some(job_id)) => {
            queue.enqueue(job_id);
            emit_changed(app, &library_id);
            tracing::info!(path = %path_text, "检测到文件变化，已加入索引队列");
        }
        Ok(None) => {}
        Err(error) => tracing::warn!(error = %error, "监听器入队失败"),
    }
}

/// 文件被删除：移除对应文档。
fn remove_deleted_document(app: &AppHandle, database: &Database, path: &Path) {
    let path_text = path.to_string_lossy().to_string();
    let library_id = database.with_connection(|connection| {
        let library_id = documents::find_library_by_path(connection, &path_text)?;
        if library_id.is_some() {
            documents::delete_document_by_path(connection, &path_text)?;
        }
        Ok(library_id)
    });

    if let Ok(Some(library_id)) = library_id {
        emit_changed(app, &library_id);
        tracing::info!(path = %path_text, "文件已删除，相关索引已清理");
    }
}

/// 通知前端某知识库数据发生变化。
fn emit_changed(app: &AppHandle, library_id: &str) {
    let _ = app.emit(
        CHANGED_EVENT,
        LibraryChanged {
            library_id: library_id.to_string(),
        },
    );
}

/// 收集需要监听的目录（已有文档的父目录，最多 32 个）。
fn watched_directories(database: &Database) -> AppResult<Vec<PathBuf>> {
    database.with_connection(|connection| {
        let mut statement = connection.prepare("SELECT DISTINCT path FROM documents LIMIT 500")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut directories: Vec<PathBuf> = Vec::new();
        for row in rows {
            let path = PathBuf::from(row?);
            let Some(parent) = path.parent() else {
                continue;
            };
            if parent.components().count() < 2 {
                continue;
            }
            let parent = parent.to_path_buf();
            if !directories.contains(&parent) {
                directories.push(parent);
            }
            if directories.len() >= 32 {
                break;
            }
        }
        Ok(directories)
    })
}

/// 每 60 秒刷新一次监听目录。
fn refresh_watches(
    watcher: &Arc<Mutex<notify::RecommendedWatcher>>,
    database: &Database,
    watched: &mut HashSet<PathBuf>,
) {
    let Ok(directories) = watched_directories(database) else {
        return;
    };
    let Ok(mut watcher) = watcher.lock() else {
        return;
    };
    for directory in directories {
        add_watch(&mut watcher, watched, &directory);
    }
}

/// 添加一个目录监听（递归模式，但跳过盘符根目录）。
fn add_watch(
    watcher: &mut notify::RecommendedWatcher,
    watched: &mut HashSet<PathBuf>,
    directory: &Path,
) {
    if watched.contains(directory) || !directory.is_dir() {
        return;
    }
    let mode = if directory.parent().is_none() {
        RecursiveMode::NonRecursive
    } else {
        RecursiveMode::Recursive
    };
    if watcher.watch(directory, mode).is_ok() {
        watched.insert(directory.to_path_buf());
    }
}
