-- 0001_init.sql —— Lumos 初始数据库结构。
--
-- 设计说明：
--   * 主键统一使用 TEXT UUID（documents / chunks 之间也是 TEXT 外键），
--     chunks 例外：FTS5 外部内容表需要 rowid，所以 chunks.id 是整数自增主键。
--   * 所有删除都通过外键 ON DELETE CASCADE 清理，避免孤儿数据。
--   * 时间戳统一为 RFC3339 UTC 文本，由 SQLite 的 strftime 生成，跨平台一致。

CREATE TABLE libraries (
    id              TEXT PRIMARY KEY NOT NULL,
    name            TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    embedding_model TEXT NOT NULL DEFAULT 'BAAI/bge-small-zh-v1.5',
    chunk_config    TEXT NOT NULL DEFAULT '{"chunk_size":512,"chunk_overlap":64}'
);

CREATE UNIQUE INDEX idx_libraries_name ON libraries (name COLLATE NOCASE);

CREATE TABLE documents (
    id           TEXT PRIMARY KEY NOT NULL,
    library_id   TEXT NOT NULL REFERENCES libraries (id) ON DELETE CASCADE,
    path         TEXT NOT NULL,
    title        TEXT NOT NULL,
    mime         TEXT NOT NULL DEFAULT '',
    size         INTEGER NOT NULL DEFAULT 0,
    page_count   INTEGER,
    status       TEXT NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending', 'indexing', 'ready', 'failed')),
    error_msg    TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    content_hash TEXT NOT NULL DEFAULT ''
);

CREATE UNIQUE INDEX idx_documents_library_path ON documents (library_id, path);
CREATE INDEX idx_documents_content_hash ON documents (content_hash);

CREATE TABLE chunks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id  TEXT NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    ordinal      INTEGER NOT NULL,
    content      TEXT NOT NULL,
    page_no      INTEGER,
    section_path TEXT,
    char_start   INTEGER NOT NULL DEFAULT 0,
    char_end     INTEGER NOT NULL DEFAULT 0,
    token_count  INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_chunks_document_ordinal ON chunks (document_id, ordinal);

-- 中文文档占大多数：trigram 分词器能覆盖 CJK 子串匹配，比 unicode61 更实用。
CREATE VIRTUAL TABLE chunks_fts USING fts5 (
    content,
    document_id UNINDEXED,
    content = 'chunks',
    content_rowid = 'id',
    tokenize = 'trigram'
);

CREATE TRIGGER chunks_ai AFTER INSERT ON chunks BEGIN
    INSERT INTO chunks_fts (rowid, content, document_id)
    VALUES (new.id, new.content, new.document_id);
END;

CREATE TRIGGER chunks_ad AFTER DELETE ON chunks BEGIN
    INSERT INTO chunks_fts (chunks_fts, rowid, content, document_id)
    VALUES ('delete', old.id, old.content, old.document_id);
END;

CREATE TRIGGER chunks_au AFTER UPDATE ON chunks BEGIN
    INSERT INTO chunks_fts (chunks_fts, rowid, content, document_id)
    VALUES ('delete', old.id, old.content, old.document_id);
    INSERT INTO chunks_fts (rowid, content, document_id)
    VALUES (new.id, new.content, new.document_id);
END;

CREATE TABLE conversations (
    id         TEXT PRIMARY KEY NOT NULL,
    library_id TEXT REFERENCES libraries (id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_conversations_library ON conversations (library_id);

CREATE TABLE messages (
    id              TEXT PRIMARY KEY NOT NULL,
    conversation_id TEXT NOT NULL REFERENCES conversations (id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK (role IN ('system', 'user', 'assistant')),
    content         TEXT NOT NULL,
    citations_json  TEXT,
    created_at      TEXT NOT NULL
);

CREATE INDEX idx_messages_conversation ON messages (conversation_id);
