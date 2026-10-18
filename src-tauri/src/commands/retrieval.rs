//! 检索相关命令：混合检索、嵌入状态查询与手动补写。

use tauri::State;

use crate::embedding::EmbeddingStatus;
use crate::retrieval::{self, DEFAULT_LIMIT, SearchMode, SearchResponse};
use crate::state::AppState;

/// 在知识库内检索相关原文片段。
///
/// # Errors
///
/// 查询为空、嵌入模型不可用（指定 vector 模式时）或数据库查询失败时返回错误。
#[tauri::command]
pub fn search_chunks(
    state: State<'_, AppState>,
    library_id: String,
    query: String,
    mode: Option<String>,
    limit: Option<usize>,
) -> Result<SearchResponse, String> {
    retrieval::search(
        &state.db,
        &state.embedder,
        &library_id,
        &query,
        SearchMode::parse(mode.as_deref()),
        limit.unwrap_or(DEFAULT_LIMIT),
    )
    .map_err(|error| error.user_message())
}

/// 查询本地嵌入模型的加载状态。
#[tauri::command]
pub fn embedding_status(state: State<'_, AppState>) -> EmbeddingStatus {
    state.embedder.status()
}

/// 手动补写一批缺失向量（诊断/修复用）。
///
/// # Errors
///
/// 数据库读写或嵌入计算失败时返回面向用户的错误。
#[tauri::command]
pub fn embedding_backfill(state: State<'_, AppState>) -> Result<usize, String> {
    retrieval::backfill(&state.db, &state.embedder, 64).map_err(|error| error.user_message())
}
