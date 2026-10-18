//! Lumos 的 Tauri 运行时入口。
//!
//! 启动顺序：解析应用数据目录 → 打开数据库 → 执行迁移 → 启动嵌入服务与摄取队列 →
//! 重新装载未完成任务 → 后台补写向量 → 注册全局状态与命令。

pub mod commands;
pub mod db;
pub mod embedding;
pub mod error;
pub mod ingest;
pub mod providers;
pub mod retrieval;
pub mod security;
pub mod state;

use std::sync::Arc;
use std::time::Duration;

use tauri::Manager;

use crate::db::repo::jobs;
use crate::embedding::EmbeddingService;
use crate::ingest::queue::IngestQueue;

/// 模型就绪后每次补写的分块数量。
const BACKFILL_BATCH: usize = 64;

/// 构建并运行 Tauri 应用。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let database = Arc::new(db::Database::open(data_dir.join("lumos.db"))?);
            database.migrate()?;

            let embedder = EmbeddingService::new(data_dir.join("models"));
            embedder.warmup();

            let queue = IngestQueue::start(
                app.handle().clone(),
                Arc::clone(&database),
                Arc::clone(&embedder),
            );
            app.manage(state::AppState {
                db: Arc::clone(&database),
                queue: Arc::clone(&queue),
                embedder: Arc::clone(&embedder),
            });

            // 重新装载上次退出时未完成的任务
            let unfinished = database.with_connection(|connection| {
                jobs::reset_running_jobs(connection)?;
                jobs::list_unfinished_jobs(connection)
            })?;
            for job in unfinished {
                queue.enqueue(job.id);
            }

            spawn_backfill(database, embedder);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::libraries::library_list,
            commands::libraries::library_create,
            commands::libraries::library_rename,
            commands::libraries::library_delete,
            commands::ingest::ingest_import_paths,
            commands::ingest::ingest_job_list,
            commands::ingest::ingest_cancel_job,
            commands::ingest::ingest_pause,
            commands::ingest::ingest_resume,
            commands::ingest::ingest_queue_state,
            commands::ingest::document_list,
            commands::ingest::document_delete,
            commands::retrieval::search_chunks,
            commands::retrieval::embedding_status,
            commands::retrieval::embedding_backfill,
            commands::providers::api_key_status,
            commands::providers::api_key_set,
            commands::providers::api_key_clear,
            commands::providers::providers_detect,
            commands::providers::settings_get,
            commands::providers::settings_set,
            commands::chat::chat_stream,
            commands::chat::chat_complete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 等嵌入模型就绪后，把历史上未向量化的分块补齐。
///
/// 模型加载失败时直接放弃；用户仍可使用关键词检索。
fn spawn_backfill(database: Arc<db::Database>, embedder: Arc<EmbeddingService>) {
    tauri::async_runtime::spawn(async move {
        loop {
            match embedder.status().state.as_str() {
                "ready" => break,
                "failed" => return,
                _ => tokio::time::sleep(Duration::from_secs(2)).await,
            }
        }

        loop {
            let task_database = Arc::clone(&database);
            let task_embedder = Arc::clone(&embedder);
            let result = tauri::async_runtime::spawn_blocking(move || {
                retrieval::backfill(&task_database, &task_embedder, BACKFILL_BATCH)
            })
            .await;

            match result {
                Ok(Ok(0)) => break,
                Ok(Ok(_)) => {}
                Ok(Err(error)) => {
                    eprintln!("补写向量失败，稍后可手动重试：{error}");
                    break;
                }
                Err(_) => break,
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });
}
