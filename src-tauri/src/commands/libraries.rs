//! 知识库相关命令：只做参数接收、调用仓储、把错误转成用户可读文案。

use tauri::State;

use crate::db::models::{Library, LibrarySummary};
use crate::db::repo::libraries as repo;
use crate::error::AppError;
use crate::state::AppState;

/// 把领域错误转换成前端可直接展示的一句话。
fn to_message(error: AppError) -> String {
    error.user_message()
}

/// 列出全部知识库及统计信息。
///
/// # Errors
///
/// 数据库读取失败时返回面向用户的错误文案。
#[tauri::command]
pub fn library_list(state: State<'_, AppState>) -> Result<Vec<LibrarySummary>, String> {
    state
        .db
        .with_connection(repo::list_libraries)
        .map_err(to_message)
}

/// 创建一个知识库。
///
/// # Errors
///
/// 名称为空、超长或重名时返回面向用户的错误文案。
#[tauri::command]
pub fn library_create(state: State<'_, AppState>, name: String) -> Result<Library, String> {
    state
        .db
        .with_connection(|connection| repo::create_library(connection, &name))
        .map_err(to_message)
}

/// 重命名一个知识库。
///
/// # Errors
///
/// 名称不合法、重名或知识库不存在时返回面向用户的错误文案。
#[tauri::command]
pub fn library_rename(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<Library, String> {
    state
        .db
        .with_connection(|connection| repo::rename_library(connection, &id, &name))
        .map_err(to_message)
}

/// 删除一个知识库（及其全部文档、分块与对话记录）。
///
/// # Errors
///
/// 知识库不存在或数据库写入失败时返回面向用户的错误文案。
#[tauri::command]
pub fn library_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .db
        .with_connection(|connection| repo::delete_library(connection, &id))
        .map_err(to_message)
}
