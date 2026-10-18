//! Lumos 的 Tauri 运行时入口。
//!
//! 启动顺序：解析应用数据目录 → 打开数据库 → 执行迁移 → 启动摄取队列 →
//! 重新装载未完成任务 → 注册全局状态与命令。

pub mod commands;
pub mod db;
pub mod error;
pub mod ingest;
pub mod state;

use std::sync::Arc;

use tauri::Manager;

use crate::db::repo::jobs;
use crate::ingest::queue::IngestQueue;

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

            let queue = IngestQueue::start(app.handle().clone(), Arc::clone(&database));
            app.manage(state::AppState {
                db: Arc::clone(&database),
                queue: Arc::clone(&queue),
            });

            // 重新装载上次退出时未完成的任务
            let unfinished = database.with_connection(|connection| {
                jobs::reset_running_jobs(connection)?;
                jobs::list_unfinished_jobs(connection)
            })?;
            for job in unfinished {
                queue.enqueue(job.id);
            }

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
