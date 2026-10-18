//! 文档解析：把各种格式统一转成「标题层级 + 正文 + 页码」的章节列表。
//!
//! 支持的格式：
//!   * PDF（pdf-extract，纯 Rust）
//!   * EPUB（自解析容器与 OPF，正文走 HTML → Markdown）
//!   * DOCX（自解析 `word/document.xml`，识别标题样式）
//!   * Markdown / 纯文本 / 常见代码文件
//!   * HTML（html2text）

mod docx;
mod epub;
mod html;
mod pdf;

use std::path::Path;

use quick_xml::events::BytesText;

use crate::error::{AppError, AppResult};
use crate::ingest::chunk::Section;

/// 解析结果。
#[derive(Debug, Clone)]
pub struct ExtractedDocument {
    /// 展示标题（优先取文档内标题，其次文件名）。
    pub title: String,
    /// MIME 类型。
    pub mime: String,
    /// 页数（仅 PDF 等分页格式）。
    pub page_count: Option<u32>,
    /// 按标题层级切好的章节。
    pub sections: Vec<Section>,
}

/// 文件格式分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Pdf,
    Epub,
    Docx,
    Markdown,
    Html,
    Text,
    Code,
}

/// 支持的扩展名（供前端文件选择器使用）。
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "pdf", "epub", "docx", "md", "markdown", "mdx", "txt", "html", "htm", "rst", "csv", "json",
    "yaml", "yml", "toml", "xml", "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "kt", "c",
    "h", "cpp", "hpp", "cs", "rb", "php", "sh", "sql", "css", "scss", "vue", "svelte", "swift",
    "lua",
];

/// 判断路径是否是 Lumos 支持索引的文件。
#[must_use]
pub fn supports(path: &Path) -> bool {
    kind_of(path).is_some()
}

/// 按扩展名判断格式。
fn kind_of(path: &Path) -> Option<Kind> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match extension.as_str() {
        "pdf" => Kind::Pdf,
        "epub" => Kind::Epub,
        "docx" => Kind::Docx,
        "md" | "markdown" | "mdx" => Kind::Markdown,
        "html" | "htm" => Kind::Html,
        "txt" | "rst" | "csv" | "json" | "yaml" | "yml" | "toml" | "xml" => Kind::Text,
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "kt" | "c" | "h" | "cpp"
        | "hpp" | "cs" | "rb" | "php" | "sh" | "sql" | "css" | "scss" | "vue" | "svelte"
        | "swift" | "lua" => Kind::Code,
        _ => return None,
    })
}

/// 解析一个文件。
///
/// # Errors
///
/// 文件不可读、格式不支持、解析失败或内容为空时返回面向用户的错误。
pub fn extract(path: &Path) -> AppResult<ExtractedDocument> {
    let kind =
        kind_of(path).ok_or_else(|| AppError::InvalidInput("暂不支持这种文件格式".to_string()))?;
    let fallback_title = file_title(path);

    match kind {
        Kind::Pdf => pdf::extract(path, &fallback_title),
        Kind::Epub => epub::extract(path, &fallback_title),
        Kind::Docx => docx::extract(path, &fallback_title),
        Kind::Html => html::extract(path, &fallback_title),
        Kind::Markdown => Ok(plain_document(
            read_text(path)?,
            file_title(path),
            mime_for(path),
            true,
        )),
        Kind::Text | Kind::Code => Ok(plain_document(
            read_text(path)?,
            fallback_title,
            mime_for(path),
            false,
        )),
    }
}

/// 读取文本文件；非 UTF-8 内容做有损转换，避免整体失败。
fn read_text(path: &Path) -> AppResult<String> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    if text.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "文件内容为空，无法建立索引".to_string(),
        ));
    }
    Ok(text)
}

/// 构造纯文本解析结果。
fn plain_document(
    text: String,
    title: String,
    mime: &'static str,
    markdown: bool,
) -> ExtractedDocument {
    let sections = if markdown {
        split_markdown(&text)
    } else {
        vec![Section {
            heading_path: String::new(),
            text,
            page_no: None,
        }]
    };

    ExtractedDocument {
        title,
        mime: mime.to_string(),
        page_count: None,
        sections,
    }
}

/// 按 Markdown 标题层级切章节。
pub(crate) fn split_markdown(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut stack: Vec<(usize, String)> = Vec::new();
    let mut buffer = String::new();

    for line in text.lines() {
        if let Some((level, heading)) = parse_heading(line) {
            flush_section(&stack, &mut buffer, &mut sections);
            while stack
                .last()
                .map(|(depth, _)| *depth >= level)
                .unwrap_or(false)
            {
                stack.pop();
            }
            stack.push((level, heading));
        } else {
            buffer.push_str(line);
            buffer.push('\n');
        }
    }

    flush_section(&stack, &mut buffer, &mut sections);
    sections
}

/// 解析 `### 标题` 形式的行。
fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let hashes = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = trimmed[hashes..].trim();
    if rest.is_empty() {
        return None;
    }
    Some((hashes, rest.to_string()))
}

/// 把缓冲区内容写入 sections。
fn flush_section(stack: &[(usize, String)], buffer: &mut String, out: &mut Vec<Section>) {
    let text = buffer.trim();
    if !text.is_empty() {
        out.push(Section {
            heading_path: stack
                .iter()
                .map(|(_, title)| title.as_str())
                .collect::<Vec<_>>()
                .join(" / "),
            text: text.to_string(),
            page_no: None,
        });
    }
    buffer.clear();
}

/// 从文件名生成标题（去扩展名）。
pub(crate) fn file_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or("未命名文档")
        .to_string()
}

/// 根据扩展名返回 MIME 类型。
pub(crate) fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("pdf") => "application/pdf",
        Some("epub") => "application/epub+zip",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("md" | "markdown" | "mdx") => "text/markdown",
        Some("html" | "htm") => "text/html",
        Some("json") => "application/json",
        _ => "text/plain",
    }
}

/// 解码 XML 文本节点（quick-xml 0.42 不再自动反转义，这里只处理常见实体）。
pub(crate) fn text_content(event: &BytesText<'_>) -> String {
    event
        .xml10_content()
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_common_formats_only() {
        assert!(supports(Path::new("a.pdf")));
        assert!(supports(Path::new("a.epub")));
        assert!(supports(Path::new("a.docx")));
        assert!(supports(Path::new("notes.md")));
        assert!(supports(Path::new("code.rs")));
        assert!(!supports(Path::new("image.png")));
        assert!(!supports(Path::new("no-extension")));
    }

    #[test]
    fn markdown_sections_follow_heading_hierarchy() {
        let text =
            "# 第一章\n引言内容。\n## 1.1 背景\n背景内容。\n### 细节\n细节内容。\n# 第二章\n结尾。";
        let sections = split_markdown(text);

        assert_eq!(sections.len(), 4);
        assert_eq!(sections[0].heading_path, "第一章");
        assert_eq!(sections[1].heading_path, "第一章 / 1.1 背景");
        assert_eq!(sections[2].heading_path, "第一章 / 1.1 背景 / 细节");
        assert_eq!(sections[3].heading_path, "第二章");
        assert!(sections[1].text.contains("背景内容"));
    }

    #[test]
    fn plain_text_becomes_single_section() {
        let document = plain_document(
            "普通内容\n第二行".to_string(),
            "笔记".to_string(),
            "text/plain",
            false,
        );
        assert_eq!(document.sections.len(), 1);
        assert_eq!(document.sections[0].heading_path, "");
    }
}
