//! HTML 文本提取：先用 html2text 转成 Markdown，再按标题层级切章节。

use std::path::Path;

use crate::error::{AppError, AppResult};
use crate::ingest::extract::{ExtractedDocument, file_title, split_markdown};

/// 提取 HTML 正文。
///
/// # Errors
///
/// 文件不可读或 HTML 无法解析时返回面向用户的错误。
pub fn extract(path: &Path, fallback_title: &str) -> AppResult<ExtractedDocument> {
    let bytes = std::fs::read(path)?;
    let markdown = html2text::from_read(bytes.as_slice(), 100)
        .map_err(|error| AppError::InvalidInput(format!("网页正文解析失败：{error}")))?;

    if markdown.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "这个网页没有可提取的正文（可能需要登录，或正文由脚本动态渲染）".to_string(),
        ));
    }

    let sections = split_markdown(&markdown);
    Ok(ExtractedDocument {
        title: if fallback_title.trim().is_empty() {
            file_title(path)
        } else {
            fallback_title.to_string()
        },
        mime: "text/html".to_string(),
        page_count: None,
        sections,
    })
}
