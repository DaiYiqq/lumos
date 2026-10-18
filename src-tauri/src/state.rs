//! 应用运行期共享状态，由 Tauri 的 `manage` 托管。

use std::sync::Arc;

use crate::db::Database;
use crate::ingest::queue::IngestQueue;

/// 全局状态：数据库 + 摄取队列。
pub struct AppState {
    /// SQLite 数据库（连接 + 迁移状态）。
    pub db: Arc<Database>,
    /// 后台摄取队列。
    pub queue: Arc<IngestQueue>,
}
