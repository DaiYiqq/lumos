//! 应用运行期共享状态，由 Tauri 的 `manage` 托管。

use std::sync::Arc;

use tracing_appender::non_blocking::WorkerGuard;

use crate::db::Database;
use crate::embedding::EmbeddingService;
use crate::ingest::queue::IngestQueue;
use crate::ingest::watcher::LibraryWatcher;

/// 全局状态：数据库 + 摄取队列 + 本地嵌入服务。
pub struct AppState {
    /// SQLite 数据库（连接 + 迁移状态）。
    pub db: Arc<Database>,
    /// 后台摄取队列。
    pub queue: Arc<IngestQueue>,
    /// 本地嵌入服务（可能仍在加载或失败）。
    pub embedder: Arc<EmbeddingService>,
    /// 文件监听器（保持存活；drop 即停止监听）。
    pub watcher: std::sync::Mutex<Option<LibraryWatcher>>,
    /// 日志文件的 guard（drop 会导致日志停止写入）。
    pub _log_guard: Option<WorkerGuard>,
}
