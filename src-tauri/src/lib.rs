//! Lumos 的 Tauri 运行时入口。
//!
//! 启动顺序：解析应用数据目录 → 打开数据库 → 执行迁移 → 注册全局状态与命令。

pub mod commands;
pub mod db;
pub mod error;
pub mod state;

use tauri::Manager;

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
            let database = db::Database::open(data_dir.join("lumos.db"))?;
            database.migrate()?;
            app.manage(state::AppState { db: database });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::libraries::library_list,
            commands::libraries::library_create,
            commands::libraries::library_rename,
            commands::libraries::library_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
