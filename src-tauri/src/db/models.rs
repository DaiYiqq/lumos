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

/// 文档处理状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    /// 已入库、等待解析。
    Pending,
    /// 正在解析/分块。
    Indexing,
    /// 可被检索。
    Ready,
    /// 解析失败。
    Failed,
}

impl DocumentStatus {
    /// 数据库存储值。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Indexing => "indexing",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

/// 文档列表项（含分块数）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    /// 文档 UUID。
    pub id: String,
    /// 所属知识库。
    pub library_id: String,
    /// 原始文件绝对路径。
    pub path: String,
    /// 展示用标题。
    pub title: String,
    /// MIME 类型。
    pub mime: String,
    /// 文件大小（字节）。
    pub size: i64,
    /// PDF 页数；非 PDF 为 null。
    pub page_count: Option<i64>,
    /// 处理状态。
    pub status: String,
    /// 失败原因（面向用户）。
    pub error_msg: Option<String>,
    /// 创建时间。
    pub created_at: String,
    /// 最后更新时间。
    pub updated_at: String,
    /// 分块数量。
    pub chunk_count: i64,
}

/// 摄取任务状态。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestJob {
    /// 任务 UUID。
    pub id: String,
    /// 所属知识库。
    pub library_id: String,
    /// 待处理文件路径。
    pub path: String,
    /// pending / running / done / failed / cancelled。
    pub status: String,
    /// 处理完成的分块数（或跳过的文件为 0）。
    pub chunk_count: i64,
    /// 结果说明（成功/失败/跳过原因）。
    pub message: Option<String>,
    /// 创建时间。
    pub created_at: String,
    /// 最后更新时间。
    pub updated_at: String,
}
