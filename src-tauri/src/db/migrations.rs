//! 版本号迁移：手写 SQL + `schema_migrations` 表，不引入 ORM。

use rusqlite::Connection;

use crate::error::AppResult;

/// 一条迁移：版本号必须单调递增，SQL 在事务内执行。
struct Migration {
    version: i64,
    sql: &'static str,
}

/// 全部迁移，按版本号升序排列。
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("../../migrations/0001_init.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("../../migrations/0002_ingest_jobs.sql"),
    },
    Migration {
        version: 3,
        sql: include_str!("../../migrations/0003_chunks_vec.sql"),
    },
    Migration {
        version: 4,
        sql: include_str!("../../migrations/0004_settings.sql"),
    },
];

/// 确保迁移表存在，然后依次应用未执行的迁移。
///
/// 每条迁移与其版本记录在同一个事务中提交，失败不会留下半套结构。
///
/// # Errors
///
/// 迁移表创建失败或任一迁移执行失败时返回 [`crate::error::AppError`]。
pub fn run(connection: &Connection) -> AppResult<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
             version    INTEGER PRIMARY KEY NOT NULL,
             applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         );",
    )?;

    for migration in MIGRATIONS {
        if is_applied(connection, migration.version)? {
            continue;
        }

        let transaction = connection.unchecked_transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            [migration.version],
        )?;
        transaction.commit()?;
    }

    Ok(())
}

/// 查询某个迁移版本是否已经应用。
fn is_applied(connection: &Connection, version: i64) -> AppResult<bool> {
    let applied = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
        [version],
        |row| row.get::<_, bool>(0),
    )?;
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn applies_all_migrations_and_is_idempotent() {
        let database = Database::open_in_memory().expect("内存库应可创建");
        database.migrate().expect("首次迁移应成功");
        database.migrate().expect("重复迁移应被跳过");

        let (count, max_version) = database
            .with_connection(|connection| {
                let count: i64 =
                    connection.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                        row.get(0)
                    })?;
                let max_version: i64 = connection.query_row(
                    "SELECT MAX(version) FROM schema_migrations",
                    [],
                    |row| row.get(0),
                )?;
                Ok((count, max_version))
            })
            .expect("查询迁移表应成功");

        assert_eq!(count, MIGRATIONS.len() as i64);
        assert_eq!(
            max_version,
            MIGRATIONS.last().map_or(0, |item| item.version)
        );
    }

    #[test]
    fn creates_core_tables_and_fts_index() {
        let database = Database::open_in_memory().expect("内存库应可创建");
        database.migrate().expect("迁移应成功");

        database
            .with_connection(|connection| {
                for table in [
                    "libraries",
                    "documents",
                    "chunks",
                    "chunks_fts",
                    "conversations",
                    "messages",
                    "schema_migrations",
                ] {
                    let exists: bool = connection.query_row(
                        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name = ?1)",
                        [table],
                        |row| row.get(0),
                    )?;
                    assert!(exists, "表 {table} 应该存在");
                }
                Ok(())
            })
            .expect("检查表结构应成功");
    }
}
