//! 摄取层：解析 → 分块 → 入库，以及持久化任务队列。

pub mod chunk;
pub mod extract;
pub mod pipeline;
pub mod queue;
