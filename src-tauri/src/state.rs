//! 应用运行期共享状态，由 Tauri 的 `manage` 托管。

use crate::db::Database;

/// 全局状态：目前只有数据库连接，后续的索引队列、Provider 也会挂在这里。
pub struct AppState {
    /// SQLite 数据库（连接 + 迁移状态）。
    pub db: Database,
}
