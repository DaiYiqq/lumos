//! 文件日志：写入应用数据目录 `logs/lumos.log`，按天滚动。
//!
//! 用户提交 issue 时可以直接附上这个文件；日志不包含 API Key、文档正文等敏感内容。

use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// 初始化日志系统；返回的 guard 必须一直存活到进程退出。
///
/// # Errors
///
/// 日志目录无法创建时返回错误（调用方可以降级为只输出到 stderr）。
pub fn init(log_dir: &Path) -> std::io::Result<WorkerGuard> {
    std::fs::create_dir_all(log_dir)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "lumos.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("lumos_lib=info,tauri=warn,ort=warn,fastembed=warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init()
        .ok();

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_log_directory_and_guard() {
        let dir = std::env::temp_dir().join(format!("lumos-log-{}", uuid::Uuid::new_v4()));
        let guard = init(&dir).expect("日志初始化应成功");
        drop(guard);
        assert!(dir.is_dir());
        std::fs::remove_dir_all(&dir).ok();
    }
}
