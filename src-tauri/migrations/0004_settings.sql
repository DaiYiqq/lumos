-- 0004_settings.sql —— 应用设置（键值对）。
--
-- 只存放非敏感配置：Ollama 地址、模型名等。
-- API Key 永远不写入数据库，统一存操作系统凭据库（见 src/security.rs）。

CREATE TABLE settings (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
