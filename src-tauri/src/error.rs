//! 应用统一错误类型。
//!
//! 设计约定：
//!   * 库层错误用 `thiserror` 分类，保留底层错误供日志排查；
//!   * 命令层只把 [`AppError::user_message`] 返回给前端，不暴露 SQL 细节；
//!   * 面向用户的文案必须是人话，不出现 `Err(...)` / SQLSTATE 这类黑话。

use thiserror::Error;

/// Lumos 后端错误。
#[derive(Debug, Error)]
pub enum AppError {
    /// SQLite 操作失败。
    #[error("数据库操作失败：{0}")]
    Database(#[from] rusqlite::Error),

    /// 文件系统操作失败（数据目录创建、日志写入等）。
    #[error("文件操作失败：{0}")]
    Io(#[from] std::io::Error),

    /// 调用方传入的参数不合法。
    #[error("{0}")]
    InvalidInput(String),

    /// 目标知识库不存在（可能已被删除）。
    #[error("找不到对应的知识库，可能已被删除")]
    LibraryNotFound,

    /// 名称冲突。
    #[error("已存在同名知识库")]
    DuplicateLibraryName,

    /// 内部状态异常（如数据库锁中毒）。
    #[error("内部状态异常：{0}")]
    Internal(String),
}

/// 库层通用返回值。
pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// 返回可以直接展示给用户的一句话。
    ///
    /// 底层数据库错误不直接透传（可能包含 SQL 与参数），只给出可操作的建议。
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Database(_) => {
                "数据库操作失败，请重试；如果持续失败，请查看日志并反馈问题".to_string()
            }
            Self::Io(_) => "读写本地文件失败，请检查磁盘空间与目录权限".to_string(),
            Self::InvalidInput(message) => message.clone(),
            Self::LibraryNotFound => "找不到对应的知识库，可能已被删除".to_string(),
            Self::DuplicateLibraryName => "已存在同名知识库，请换一个名字".to_string(),
            Self::Internal(_) => "应用内部状态异常，请重启 Lumos 后重试".to_string(),
        }
    }
}
