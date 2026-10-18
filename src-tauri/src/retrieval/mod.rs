//! 混合检索：向量 KNN + BM25 关键字，用 RRF（Reciprocal Rank Fusion）融合。
//!
//! RRF 只依赖名次而不依赖分数尺度，因此不需要把「距离」和「bm25 分数」归一化，
//! 这也是它适合混合检索的原因。常数 `k = 60` 取自 RRF 原始论文的常用取值。

use std::collections::HashMap;

use serde::Serialize;

use crate::db::Database;
use crate::db::repo::chunks::{self, ChunkDetail};
use crate::embedding::EmbeddingService;
use crate::error::{AppError, AppResult};

/// RRF 平滑常数。
pub const RRF_K: f64 = 60.0;

/// 默认返回条数。
pub const DEFAULT_LIMIT: usize = 8;

/// 检索模式（诊断与降级共用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// 向量 + 关键词融合。
    Hybrid,
    /// 仅向量。
    Vector,
    /// 仅关键词。
    Keyword,
}

impl SearchMode {
    /// 从命令参数解析；未知或缺省时使用混合模式。
    #[must_use]
    pub fn parse(value: Option<&str>) -> Self {
        match value.map(str::to_ascii_lowercase).as_deref() {
            Some("vector") => Self::Vector,
            Some("keyword") => Self::Keyword,
            _ => Self::Hybrid,
        }
    }

    /// 返回稳定的字符串标识。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hybrid => "hybrid",
            Self::Vector => "vector",
            Self::Keyword => "keyword",
        }
    }
}

/// 单条检索命中。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    /// 分块 ID。
    pub chunk_id: i64,
    /// 文档 ID。
    pub document_id: String,
    /// 文档标题。
    pub document_title: String,
    /// 文档原始路径。
    pub document_path: String,
    /// 页码。
    pub page_no: Option<i64>,
    /// 章节路径。
    pub section_path: Option<String>,
    /// 分块正文。
    pub content: String,
    /// 融合后的相关性分数（越大越相关）。
    pub score: f64,
    /// 在向量检索中的名次（从 1 开始）。
    pub vector_rank: Option<usize>,
    /// 在 BM25 检索中的名次（从 1 开始）。
    pub keyword_rank: Option<usize>,
}

/// 检索响应。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    /// 原始查询。
    pub query: String,
    /// 调用方请求的模式。
    pub requested_mode: String,
    /// 实际使用的模式；模型不可用时混合模式会降级为 `keyword`。
    pub mode: String,
    /// 向量检索是否参与。
    pub vector_enabled: bool,
    /// 命中列表。
    pub hits: Vec<SearchHit>,
}

impl SearchResponse {
    /// 是否处于「离线关键词」降级模式（前端据此提示用户）。
    #[must_use]
    pub fn is_keyword_fallback(&self) -> bool {
        self.requested_mode != "keyword" && self.mode == "keyword"
    }
}

/// 执行检索。
///
/// # Errors
///
/// 查询为空、指定了向量模式但模型不可用，或数据库查询失败时返回错误。
pub fn search(
    database: &Database,
    embedder: &EmbeddingService,
    library_id: &str,
    query: &str,
    mode: SearchMode,
    limit: usize,
) -> AppResult<SearchResponse> {
    let query = query.trim();
    if query.is_empty() {
        return Err(AppError::InvalidInput("请输入要检索的内容".to_string()));
    }
    let limit = limit.clamp(1, 50);
    let vector_ready = embedder.is_ready();

    if mode == SearchMode::Vector && !vector_ready {
        return Err(AppError::InvalidInput(
            "本地嵌入模型尚未就绪，暂时只能用关键词检索".to_string(),
        ));
    }

    let mut vector_ids: Vec<i64> = Vec::new();
    if mode != SearchMode::Keyword && vector_ready {
        let vector = embedder.embed_query(query)?;
        let hits = database.with_connection(|connection| {
            chunks::vector_search(connection, &vector, library_id, limit * 2)
        })?;
        vector_ids = hits.into_iter().map(|hit| hit.chunk_id).collect();
    }

    let mut keyword_ids: Vec<i64> = Vec::new();
    if mode != SearchMode::Vector {
        let hits = database.with_connection(|connection| {
            chunks::keyword_search(connection, query, library_id, limit * 2)
        })?;
        keyword_ids = hits.into_iter().map(|hit| hit.chunk_id).collect();
    }

    let mode_used = match mode {
        SearchMode::Hybrid => {
            if vector_ready {
                "hybrid"
            } else {
                "keyword"
            }
        }
        SearchMode::Vector => "vector",
        SearchMode::Keyword => "keyword",
    };

    let fused = match mode_used {
        "hybrid" => rrf_fuse(&vector_ids, &keyword_ids, limit),
        "vector" => vector_ids
            .iter()
            .take(limit)
            .enumerate()
            .map(|(index, id)| (*id, 1.0 / (RRF_K + (index + 1) as f64)))
            .collect(),
        _ => keyword_ids
            .iter()
            .take(limit)
            .enumerate()
            .map(|(index, id)| (*id, 1.0 / (RRF_K + (index + 1) as f64)))
            .collect(),
    };

    let vector_ranks: HashMap<i64, usize> = vector_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index + 1))
        .collect();
    let keyword_ranks: HashMap<i64, usize> = keyword_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index + 1))
        .collect();

    let ordered_ids: Vec<i64> = fused.iter().map(|(id, _)| *id).collect();
    let details =
        database.with_connection(|connection| chunks::chunk_details(connection, &ordered_ids))?;
    let details_by_id: HashMap<i64, ChunkDetail> = details
        .into_iter()
        .map(|detail| (detail.chunk_id, detail))
        .collect();

    let mut hits = Vec::new();
    for (chunk_id, score) in fused {
        let Some(detail) = details_by_id.get(&chunk_id) else {
            continue;
        };
        hits.push(SearchHit {
            chunk_id: detail.chunk_id,
            document_id: detail.document_id.clone(),
            document_title: detail.document_title.clone(),
            document_path: detail.document_path.clone(),
            page_no: detail.page_no,
            section_path: detail.section_path.clone(),
            content: detail.content.clone(),
            score,
            vector_rank: vector_ranks.get(&chunk_id).copied(),
            keyword_rank: keyword_ranks.get(&chunk_id).copied(),
        });
    }

    Ok(SearchResponse {
        query: query.to_string(),
        requested_mode: mode.as_str().to_string(),
        mode: mode_used.to_string(),
        vector_enabled: vector_ready && mode_used != "keyword",
        hits,
    })
}

/// 用 RRF 融合两路检索结果。
///
/// 每个分块的分数为 `Σ 1 / (k + rank)`，同一分块在两路中都出现时分数相加。
#[must_use]
pub fn rrf_fuse(vector_ids: &[i64], keyword_ids: &[i64], limit: usize) -> Vec<(i64, f64)> {
    let mut scores: HashMap<i64, f64> = HashMap::new();

    for (index, chunk_id) in vector_ids.iter().enumerate() {
        *scores.entry(*chunk_id).or_insert(0.0) += 1.0 / (RRF_K + (index + 1) as f64);
    }
    for (index, chunk_id) in keyword_ids.iter().enumerate() {
        *scores.entry(*chunk_id).or_insert(0.0) += 1.0 / (RRF_K + (index + 1) as f64);
    }

    let mut fused: Vec<(i64, f64)> = scores.into_iter().collect();
    fused.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });
    fused.truncate(limit);
    fused
}

/// 为尚未向量化的分块补写向量，返回本轮写入数量（0 表示已全部完成）。
///
/// # Errors
///
/// 数据库读写或嵌入计算出错时返回错误。
pub fn backfill(
    database: &Database,
    embedder: &EmbeddingService,
    batch_size: usize,
) -> AppResult<usize> {
    if !embedder.is_ready() {
        return Ok(0);
    }

    let pending = database.with_connection(|connection| {
        chunks::list_chunks_missing_vectors(connection, batch_size)
    })?;
    if pending.is_empty() {
        return Ok(0);
    }

    let texts: Vec<String> = pending.iter().map(|chunk| chunk.content.clone()).collect();
    let vectors = embedder.embed_documents(&texts)?;
    if vectors.len() != pending.len() {
        return Err(AppError::Internal(
            "嵌入结果数量与分块数量不一致".to_string(),
        ));
    }

    let pairs: Vec<(i64, Vec<f32>)> = pending
        .iter()
        .zip(vectors)
        .map(|(chunk, vector)| (chunk.id, vector))
        .collect();
    database.with_connection(|connection| chunks::insert_vectors(connection, &pairs))?;
    Ok(pairs.len())
}

#[cfg(test)]
mod tests;
