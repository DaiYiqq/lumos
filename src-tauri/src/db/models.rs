//! 数据库模型：与前端 JSON 字段一一对应（camelCase）。

use serde::Serialize;

/// 知识库基础信息。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    /// 知识库 UUID。
    pub id: String,
    /// 用户可见名称，全局唯一（忽略大小写）。
    pub name: String,
    /// 创建时间（RFC3339 UTC）。
    pub created_at: String,
    /// 最后更新时间（RFC3339 UTC）。
    pub updated_at: String,
    /// 该库使用的嵌入模型标识。
    pub embedding_model: String,
    /// 分块配置（JSON 字符串）。
    pub chunk_config: String,
}

/// 知识库列表项：基础信息 + 统计字段。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySummary {
    /// 基础信息。
    #[serde(flatten)]
    pub library: Library,
    /// 文档数量。
    pub document_count: i64,
    /// 分块数量。
    pub chunk_count: i64,
    /// 文档体积总和（字节）。
    pub size_bytes: i64,
}
