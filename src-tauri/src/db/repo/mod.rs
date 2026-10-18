//! 仓储层：每个领域一个模块，只依赖 `rusqlite::Connection`，便于单元测试。

pub mod documents;
pub mod jobs;
pub mod libraries;
