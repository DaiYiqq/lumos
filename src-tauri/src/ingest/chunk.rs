//! 层级感知分块。
//!
//! 设计目标：
//!   * 优先按标题层级切分（解析器已把文档拆成 [`Section`]），章节内部再按段落打包；
//!   * 目标块大小约 512 token，块间重叠约 64 token，避免关键句被切断；
//!   * 每个块保留页码、章节路径与原文字符区间，供引用溯源使用。

/// 解析器输出的一个章节片段。
#[derive(Debug, Clone)]
pub struct Section {
    /// 标题层级路径，例如 `第三章 / 3.2 违约金`。
    pub heading_path: String,
    /// 章节正文（可含多段）。
    pub text: String,
    /// 页码（仅 PDF 等分页格式）。
    pub page_no: Option<u32>,
}

/// 分块结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// 分块正文。
    pub content: String,
    /// 章节路径。
    pub section_path: Option<String>,
    /// 页码。
    pub page_no: Option<i64>,
    /// 在原文中的起始字符偏移。
    pub char_start: i64,
    /// 在原文中的结束字符偏移。
    pub char_end: i64,
    /// 估算 token 数。
    pub token_count: i64,
}

/// 默认分块大小（token）。
pub const DEFAULT_CHUNK_SIZE: usize = 512;
/// 默认重叠大小（token）。
pub const DEFAULT_CHUNK_OVERLAP: usize = 64;

/// 估算文本 token 数。
///
/// 中文（CJK）按「1 字符 ≈ 1 token」估算，拉丁字母按「4 字符 ≈ 1 token」估算。
/// 这是为分块服务的保守近似，不追求与具体 tokenizer 完全一致。
#[must_use]
pub fn estimate_tokens(text: &str) -> usize {
    let mut latin = 0usize;
    let mut cjk = 0usize;

    for character in text.chars() {
        if is_cjk(character) {
            cjk += 1;
        } else {
            latin += 1;
        }
    }

    let latin_tokens = latin.div_ceil(4);
    (latin_tokens + cjk).max(if text.is_empty() { 0 } else { 1 })
}

/// 判断是否为 CJK 字符（中日韩统一表意文字、假名、谚文）。
fn is_cjk(character: char) -> bool {
    matches!(character as u32,
        0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF)
}

/// 把章节切分成块。
#[must_use]
pub fn chunk_sections(sections: &[Section], chunk_size: usize, overlap: usize) -> Vec<Chunk> {
    let chunk_size = chunk_size.max(64);
    let overlap = overlap.min(chunk_size / 2);
    let mut chunks = Vec::new();

    for section in sections {
        if section.text.trim().is_empty() {
            continue;
        }
        split_section(section, chunk_size, overlap, &mut chunks);
    }

    chunks
}

/// 处理单个章节。
fn split_section(section: &Section, chunk_size: usize, overlap: usize, out: &mut Vec<Chunk>) {
    let mut current = String::new();
    let mut current_start = 0usize;
    let mut cursor = 0usize;

    for raw_line in section.text.split_inclusive('\n') {
        let line = raw_line.trim();
        let line_start = cursor + (raw_line.len() - raw_line.trim_start().len());
        let line_end = line_start + line.len();
        cursor += raw_line.len();

        if line.is_empty() {
            continue;
        }

        if estimate_tokens(line) > chunk_size {
            flush(out, &mut current, &mut current_start, section, line_start);
            for (offset, piece) in split_oversized(line, chunk_size) {
                let start = line_start + offset;
                push_piece(out, piece, start, section);
            }
            current_start = line_end;
            continue;
        }

        let combined = estimate_tokens(&current) + estimate_tokens(line);
        if !current.trim().is_empty() && combined > chunk_size {
            let tail = tail_by_tokens(&current, overlap);
            let flush_end = current_start + current.len();
            flush(out, &mut current, &mut current_start, section, line_start);
            if !tail.is_empty() {
                current_start = flush_end.saturating_sub(tail.len());
                current.push_str(&tail);
            }
        }

        if current.is_empty() {
            current_start = line_start;
        } else {
            current.push('\n');
        }
        current.push_str(line);
    }

    flush(out, &mut current, &mut current_start, section, cursor);
}

/// 把当前缓冲区写成一个块并清空。
fn flush(
    out: &mut Vec<Chunk>,
    current: &mut String,
    current_start: &mut usize,
    section: &Section,
    next_start: usize,
) {
    let content = current.trim();
    if !content.is_empty() {
        let leading = current.len() - current.trim_start().len();
        let start = *current_start + leading;
        out.push(Chunk {
            content: content.to_string(),
            section_path: non_empty(&section.heading_path),
            page_no: section.page_no.map(i64::from),
            char_start: i64::try_from(start).unwrap_or(i64::MAX),
            char_end: i64::try_from(start + content.len()).unwrap_or(i64::MAX),
            token_count: i64::try_from(estimate_tokens(content)).unwrap_or(i64::MAX),
        });
    }
    current.clear();
    *current_start = next_start;
}

/// 把超长段落切出的一片直接写成一个块。
fn push_piece(out: &mut Vec<Chunk>, piece: &str, start: usize, section: &Section) {
    let content = piece.trim();
    if content.is_empty() {
        return;
    }
    let leading = piece.len() - piece.trim_start().len();
    let start = start + leading;
    out.push(Chunk {
        content: content.to_string(),
        section_path: non_empty(&section.heading_path),
        page_no: section.page_no.map(i64::from),
        char_start: i64::try_from(start).unwrap_or(i64::MAX),
        char_end: i64::try_from(start + content.len()).unwrap_or(i64::MAX),
        token_count: i64::try_from(estimate_tokens(content)).unwrap_or(i64::MAX),
    });
}

/// 单个超长段落按 token 预算切片。
fn split_oversized(text: &str, chunk_size: usize) -> Vec<(usize, &str)> {
    // 以「四分之一个 token」为单位计数：1 个拉丁字符 = 1 单位，1 个 CJK 字符 = 4 单位。
    let budget = chunk_size.saturating_mul(4);
    let mut pieces = Vec::new();
    let mut start = 0usize;
    let mut units = 0usize;

    for (offset, character) in text.char_indices() {
        units += if is_cjk(character) { 4 } else { 1 };
        if units >= budget && offset > start {
            pieces.push((start, &text[start..offset]));
            start = offset;
            units = 0;
        }
    }

    if start < text.len() {
        pieces.push((start, &text[start..]));
    }
    pieces
}

/// 取文本尾部约 `max_tokens` 个 token 的内容，用于块间重叠。
fn tail_by_tokens(text: &str, max_tokens: usize) -> String {
    if max_tokens == 0 || text.is_empty() {
        return String::new();
    }

    let budget = max_tokens.saturating_mul(4);
    let mut units = 0usize;
    let mut start = text.len();

    for (offset, character) in text.char_indices().rev() {
        units += if is_cjk(character) { 4 } else { 1 };
        start = offset;
        if units > budget {
            break;
        }
    }

    text[start..].trim_start().to_string()
}

/// 空字符串转 `None`。
fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(text: &str, heading: &str, page: Option<u32>) -> Section {
        Section {
            heading_path: heading.to_string(),
            text: text.to_string(),
            page_no: page,
        }
    }

    #[test]
    fn estimates_tokens_for_latin_and_cjk() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abc"), 1);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
        assert_eq!(estimate_tokens("违约金上限"), 5);
        assert_eq!(estimate_tokens("违约金 abcd"), 5);
    }

    #[test]
    fn splits_long_section_and_keeps_overlap() {
        let paragraph = "这是一段用于测试分块的正文内容，包含足够的长度来触发多次切分。".repeat(60);
        let sections = vec![section(&paragraph, "第一章", Some(3))];
        let chunks = chunk_sections(&sections, 256, 32);

        assert!(chunks.len() > 2, "长文本应被切成多块");
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            assert!(
                chunk.token_count <= 300,
                "块不应显著超过预算: {}",
                chunk.token_count
            );
            assert_eq!(chunk.section_path.as_deref(), Some("第一章"));
            assert_eq!(chunk.page_no, Some(3));
            assert!(chunk.char_start < chunk.char_end);
        }
    }

    #[test]
    fn oversized_paragraph_is_sliced() {
        let huge = "超长段落".repeat(2000);
        let sections = vec![section(&huge, "", None)];
        let chunks = chunk_sections(&sections, 128, 16);

        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.token_count <= 160);
        }
    }

    #[test]
    fn empty_sections_are_ignored() {
        let sections = vec![
            section("   \n  ", "空章节", None),
            section("有效内容", "有内容", None),
        ];
        let chunks = chunk_sections(&sections, 256, 32);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "有效内容");
        assert_eq!(chunks[0].section_path.as_deref(), Some("有内容"));
    }

    #[test]
    fn char_offsets_stay_inside_section_text() {
        let text = "第一段内容。\n第二段内容更长一些。\n第三段。";
        let sections = vec![section(text, "段落", None)];
        let chunks = chunk_sections(&sections, 64, 8);

        for chunk in &chunks {
            let start = usize::try_from(chunk.char_start).expect("偏移应为非负");
            let end = usize::try_from(chunk.char_end).expect("偏移应为非负");
            assert!(end <= text.len());
            assert!(start < end);
        }
    }
}
