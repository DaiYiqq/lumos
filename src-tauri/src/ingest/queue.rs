//! 持久化摄取队列。
//!
//! 任务本身写在 SQLite 的 `ingest_jobs` 表里，进程退出不丢；
//! 队列只在内存中保存「待执行的任务 ID 列表」，启动时从数据库重新装载。
//! 工作线程串行处理任务，支持全局暂停（文件之间生效）与单任务取消。

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::db::Database;
use crate::db::repo::jobs;
use crate::embedding::EmbeddingService;
use crate::error::AppError;
use crate::ingest::pipeline;

/// 推送给前端的进度事件名。
pub const PROGRESS_EVENT: &str = "ingest://progress";

/// 进度事件负载。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestProgress {
    /// 任务 ID。
    pub job_id: String,
    /// 知识库 ID。
    pub library_id: String,
    /// 文件路径。
    pub path: String,
    /// 任务状态。
    pub status: String,
    /// 已写入分块数。
    pub chunk_count: i64,
    /// 结果说明。
    pub message: Option<String>,
}

/// 摄取队列句柄。
pub struct IngestQueue {
    sender: mpsc::UnboundedSender<String>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<Mutex<HashSet<String>>>,
}

impl IngestQueue {
    /// 启动后台工作线程。
    #[must_use]
    pub fn start(
        app: AppHandle,
        database: Arc<Database>,
        embedder: Arc<EmbeddingService>,
    ) -> Arc<Self> {
        let (sender, mut receiver) = mpsc::unbounded_channel::<String>();
        let paused = Arc::new(AtomicBool::new(false));
        let cancelled: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        let worker_paused = Arc::clone(&paused);
        let worker_cancelled = Arc::clone(&cancelled);
        let worker_database = Arc::clone(&database);
        let worker_embedder = Arc::clone(&embedder);

        tauri::async_runtime::spawn(async move {
            while let Some(job_id) = receiver.recv().await {
                while worker_paused.load(Ordering::Relaxed) {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }

                let job = match worker_database
                    .with_connection(|connection| jobs::get_job(connection, &job_id))
                {
                    Ok(Some(job)) => job,
                    _ => continue,
                };

                if is_cancelled(&worker_cancelled, &job_id) {
                    finish_job(
                        &app,
                        &worker_database,
                        &job,
                        "cancelled",
                        0,
                        Some("任务已取消"),
                    );
                    remove_cancelled(&worker_cancelled, &job_id);
                    continue;
                }

                let _ = worker_database.with_connection(|connection| {
                    jobs::update_job(connection, &job_id, "running", 0, None)
                });
                emit_progress(&app, &job, "running", 0, None);

                let task_database = Arc::clone(&worker_database);
                let task_cancelled = Arc::clone(&worker_cancelled);
                let task_embedder = Arc::clone(&worker_embedder);
                let task_job_id = job_id.clone();
                let library_id = job.library_id.clone();
                let path = job.path.clone();
                let outcome = tauri::async_runtime::spawn_blocking(move || {
                    let check = || is_cancelled(&task_cancelled, &task_job_id);
                    pipeline::ingest_file(
                        &task_database,
                        &library_id,
                        std::path::Path::new(&path),
                        check,
                        Some(&task_embedder),
                    )
                })
                .await;

                match outcome {
                    Ok(Ok(outcome)) => {
                        finish_job(
                            &app,
                            &worker_database,
                            &job,
                            "done",
                            i64::try_from(outcome.chunk_count).unwrap_or(i64::MAX),
                            outcome.message.as_deref(),
                        );
                    }
                    Ok(Err(AppError::Cancelled)) => {
                        finish_job(
                            &app,
                            &worker_database,
                            &job,
                            "cancelled",
                            0,
                            Some("任务已取消"),
                        );
                    }
                    Ok(Err(error)) => {
                        let message = error.user_message();
                        finish_job(&app, &worker_database, &job, "failed", 0, Some(&message));
                    }
                    Err(error) => {
                        let message = format!("后台任务异常退出：{error}");
                        finish_job(&app, &worker_database, &job, "failed", 0, Some(&message));
                    }
                }

                remove_cancelled(&worker_cancelled, &job_id);
            }
        });

        Arc::new(Self {
            sender,
            paused,
            cancelled,
        })
    }

    /// 把任务加入内存队列。
    pub fn enqueue(&self, job_id: impl Into<String>) {
        let _ = self.sender.send(job_id.into());
    }

    /// 暂停队列（当前文件处理完后暂停）。
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    /// 恢复队列。
    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    /// 队列是否处于暂停状态。
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    /// 请求取消一个任务；正在运行的任务会在下一个检查点退出。
    pub fn cancel(&self, job_id: &str) {
        if let Ok(mut set) = self.cancelled.lock() {
            set.insert(job_id.to_string());
        }
    }
}

/// 查询取消集合。
fn is_cancelled(cancelled: &Arc<Mutex<HashSet<String>>>, job_id: &str) -> bool {
    cancelled
        .lock()
        .map(|set| set.contains(job_id))
        .unwrap_or(false)
}

/// 从取消集合中移除（任务已结束）。
fn remove_cancelled(cancelled: &Arc<Mutex<HashSet<String>>>, job_id: &str) {
    if let Ok(mut set) = cancelled.lock() {
        set.remove(job_id);
    }
}

/// 写入最终状态并推送事件。
fn finish_job(
    app: &AppHandle,
    database: &Database,
    job: &crate::db::models::IngestJob,
    status: &str,
    chunk_count: i64,
    message: Option<&str>,
) {
    let _ = database.with_connection(|connection| {
        jobs::update_job(connection, &job.id, status, chunk_count, message)
    });
    emit_progress(app, job, status, chunk_count, message);
}

/// 推送进度事件（前端据此刷新文档列表与进度条）。
fn emit_progress(
    app: &AppHandle,
    job: &crate::db::models::IngestJob,
    status: &str,
    chunk_count: i64,
    message: Option<&str>,
) {
    let payload = IngestProgress {
        job_id: job.id.clone(),
        library_id: job.library_id.clone(),
        path: job.path.clone(),
        status: status.to_string(),
        chunk_count,
        message: message.map(str::to_string),
    };
    let _ = app.emit(PROGRESS_EVENT, payload);
}
