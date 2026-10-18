//! PDF 文本提取。
//!
//! 使用纯 Rust 的 `pdf-extract`。扫描件没有文本层，会得到空内容，
//! 这里转成面向用户的提示，而不是把底层错误抛给前端。

use std::path::Path;

use crate::error::{AppError, AppResult};
use crate::ingest::chunk::Section;
use crate::ingest::extract::{ExtractedDocument, file_title};

/// 提取 PDF 文本。
///
/// # Errors
///
/// 文件损坏、加密或没有文本层时返回面向用户的错误。
pub fn extract(path: &Path, fallback_title: &str) -> AppResult<ExtractedDocument> {
    let text = pdf_extract::extract_text(path).map_err(|error| {
        AppError::InvalidInput(format!("PDF 解析失败（文件可能损坏或被加密）：{error}"))
    })?;

    if text.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "PDF 解析失败：这份文件可能是扫描件，没有可提取的文字层".to_string(),
        ));
    }

    let pages: Vec<&str> = text.split('\u{0C}').collect();
    let mut sections = Vec::new();

    if pages.len() > 1 {
        for (index, page) in pages.iter().enumerate() {
            if page.trim().is_empty() {
                continue;
            }
            sections.push(Section {
                heading_path: String::new(),
                text: page.trim().to_string(),
                page_no: Some(u32::try_from(index + 1).unwrap_or(u32::MAX)),
            });
        }
    } else {
        sections.push(Section {
            heading_path: String::new(),
            text: text.trim().to_string(),
            page_no: None,
        });
    }

    let title = if fallback_title.trim().is_empty() {
        file_title(path)
    } else {
        fallback_title.to_string()
    };

    Ok(ExtractedDocument {
        title,
        mime: "application/pdf".to_string(),
        page_count: u32::try_from(pages.len()).ok(),
        sections,
    })
}
