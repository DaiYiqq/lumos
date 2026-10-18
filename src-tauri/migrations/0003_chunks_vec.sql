-- 0003_chunks_vec.sql —— sqlite-vec 向量表。
--
-- 维度固定为 512：BGE-small-zh-v1.5 的输出维度。
-- 如果未来更换嵌入模型，需要新增迁移并重建该表（数据库里会记录 embedding_model）。
-- 该虚拟表的创建依赖 sqlite-vec 扩展，扩展在打开连接前已注册（见 db/mod.rs）。

CREATE VIRTUAL TABLE chunks_vec USING vec0(
    chunk_id  INTEGER PRIMARY KEY,
    embedding FLOAT[512]
);

-- 虚拟表没有外键，删除分块时手工清理对应向量，避免留下孤儿向量。
CREATE TRIGGER chunks_vec_ad AFTER DELETE ON chunks BEGIN
    DELETE FROM chunks_vec WHERE chunk_id = old.id;
END;
