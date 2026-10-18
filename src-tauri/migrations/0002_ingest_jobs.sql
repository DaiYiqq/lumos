-- 0002_ingest_jobs.sql —— 摄取任务队列表。
--
-- 任务状态持久化在 SQLite 中：窗口关闭、应用重启后，未完成的任务会重新入队，
-- 已完成的记录保留用于展示最近一次的导入结果。

CREATE TABLE ingest_jobs (
    id          TEXT PRIMARY KEY NOT NULL,
    library_id  TEXT NOT NULL REFERENCES libraries (id) ON DELETE CASCADE,
    path        TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'running', 'done', 'failed', 'cancelled')),
    chunk_count INTEGER NOT NULL DEFAULT 0,
    message     TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX idx_ingest_jobs_library ON ingest_jobs (library_id, status);
CREATE INDEX idx_ingest_jobs_status ON ingest_jobs (status);
