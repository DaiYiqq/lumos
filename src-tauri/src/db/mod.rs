//! 数据库连接、迁移与仓储层。
//!
//! 同一时刻只有一个写连接，由 [`Database`] 内部的互斥锁串行化；
//! 开启 WAL 后读操作不会阻塞写操作，足以支撑单机知识库场景。

pub mod migrations;
pub mod models;
pub mod repo;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

/// SQLite 连接包装器。
pub struct Database {
    connection: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    /// 打开（或创建）磁盘上的数据库文件。
    ///
    /// # Errors
    ///
    /// 目录无法创建、文件无法打开或 PRAGMA 设置失败时返回 [`AppError`]。
    pub fn open(path: impl AsRef<Path>) -> AppResult<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(&path)?;
        Self::configure(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
            path,
        })
    }

    /// 内存数据库，仅用于单元测试与快速验证。
    ///
    /// # Errors
    ///
    /// PRAGMA 设置失败时返回 [`AppError`]。
    pub fn open_in_memory() -> AppResult<Self> {
        let connection = Connection::open_in_memory()?;
        Self::configure(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
            path: PathBuf::from(":memory:"),
        })
    }

    /// 设置连接级 PRAGMA：外键、WAL、忙等待、同步级别。
    fn configure(connection: &Connection) -> AppResult<()> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )?;
        Ok(())
    }

    /// 执行所有尚未应用的迁移。
    ///
    /// # Errors
    ///
    /// 迁移 SQL 执行失败或迁移表无法写入时返回 [`AppError`]。
    pub fn migrate(&self) -> AppResult<()> {
        self.with_connection(migrations::run)
    }

    /// 在连接锁内执行一段数据库操作。
    ///
    /// # Errors
    ///
    /// 连接锁中毒或 `operation` 返回错误时返回 [`AppError`]。
    pub fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> AppResult<T>,
    ) -> AppResult<T> {
        let guard = self
            .connection
            .lock()
            .map_err(|_| AppError::Internal("数据库连接锁已中毒".to_string()))?;
        operation(&guard)
    }

    /// 数据库文件的绝对路径；内存库返回 `:memory:`。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}
