//! DOCX 文本提取。
//!
//! 不依赖完整 Office 解析库：直接解开 zip，读取 `word/document.xml`，
//! 按 `w:p`（段落）与 `w:pStyle`（段落样式）重建「标题 + 正文」结构。

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::error::{AppError, AppResult};
use crate::ingest::chunk::Section;
use crate::ingest::extract::{ExtractedDocument, file_title};

/// 提取 DOCX 正文。
///
/// # Errors
///
/// 文件不是合法 DOCX、XML 损坏或没有文字内容时返回面向用户的错误。
pub fn extract(path: &Path, fallback_title: &str) -> AppResult<ExtractedDocument> {
    let xml = read_zip_entry(path, "word/document.xml")?;
    let paragraphs = parse_paragraphs(&xml)?;

    let mut sections = Vec::new();
    let mut stack: Vec<(usize, String)> = Vec::new();
    let mut buffer = String::new();
    let mut title: Option<String> = None;

    for (style, text) in paragraphs {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(level) = heading_level(&style) {
            flush(&stack, &mut buffer, &mut sections);
            while stack
                .last()
                .map(|(depth, _)| *depth >= level)
                .unwrap_or(false)
            {
                stack.pop();
            }
            stack.push((level, trimmed.to_string()));
            if title.is_none() && level == 1 {
                title = Some(trimmed.to_string());
            }
        } else {
            buffer.push_str(trimmed);
            buffer.push('\n');
        }
    }
    flush(&stack, &mut buffer, &mut sections);

    if sections.is_empty() {
        return Err(AppError::InvalidInput(
            "Word 文档里没有可提取的文字（可能只包含图片）".to_string(),
        ));
    }

    Ok(ExtractedDocument {
        title: title.unwrap_or_else(|| {
            if fallback_title.trim().is_empty() {
                file_title(path)
            } else {
                fallback_title.to_string()
            }
        }),
        mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        page_count: None,
        sections,
    })
}

/// 读取 zip 内的一个文本条目。
fn read_zip_entry(path: &Path, entry: &str) -> AppResult<String> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .map_err(|error| AppError::InvalidInput(format!("无法打开 Word 文档：{error}")))?;
    let mut entry = archive
        .by_name(entry)
        .map_err(|_| AppError::InvalidInput("这份 Word 文档结构不完整".to_string()))?;

    let mut xml = String::new();
    entry.read_to_string(&mut xml)?;
    Ok(xml)
}

/// 提取每个段落的（样式, 文本）。
fn parse_paragraphs(xml: &str) -> AppResult<Vec<(String, String)>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buffer = Vec::new();
    let mut paragraphs = Vec::new();
    let mut current_text = String::new();
    let mut current_style = String::new();
    let mut in_paragraph = false;
    let mut in_text = false;

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) => match event.local_name().as_ref() {
                "p" => {
                    in_paragraph = true;
                    current_text.clear();
                    current_style.clear();
                }
                "pStyle" => {
                    if let Some(value) = attribute(&event, "val") {
                        current_style = value;
                    }
                }
                "t" => in_text = true,
                _ => {}
            },
            Ok(Event::Empty(event)) => {
                if event.local_name().as_ref() == "pStyle"
                    && let Some(value) = attribute(&event, "val")
                {
                    current_style = value;
                }
            }
            Ok(Event::End(event)) => match event.local_name().as_ref() {
                "p" => {
                    if in_paragraph {
                        paragraphs.push((current_style.clone(), current_text.clone()));
                        in_paragraph = false;
                    }
                }
                "t" => in_text = false,
                _ => {}
            },
            Ok(Event::Text(event)) => {
                if in_text {
                    current_text.push_str(&crate::ingest::extract::text_content(&event));
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                return Err(AppError::InvalidInput(format!(
                    "Word 文档解析失败：{error}"
                )));
            }
        }
        buffer.clear();
    }

    Ok(paragraphs)
}

/// 读取 XML 属性值（按本地名匹配，忽略命名空间前缀）。
fn attribute(event: &BytesStart<'_>, name: &str) -> Option<String> {
    for attribute in event.attributes().flatten() {
        if attribute.key.local_name().as_ref() == name {
            return attribute
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()
                .map(|value| value.into_owned());
        }
    }
    None
}

/// 把样式名映射为标题层级。
fn heading_level(style: &str) -> Option<usize> {
    let normalized = style.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }

    let digits: String = normalized
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    let level = digits.parse::<usize>().ok();
    let looks_like_heading = normalized.contains("heading")
        || normalized.contains("title")
        || normalized.contains("标题");

    match (looks_like_heading, level) {
        (true, Some(level @ 1..=6)) => Some(level),
        (true, None) => Some(1),
        (false, Some(level @ 2..=6)) => Some(level),
        _ => None,
    }
}

/// 把缓冲区内容写入 sections。
fn flush(stack: &[(usize, String)], buffer: &mut String, out: &mut Vec<Section>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_heading_styles_to_levels() {
        assert_eq!(heading_level("Heading1"), Some(1));
        assert_eq!(heading_level("Heading 2"), Some(2));
        assert_eq!(heading_level("标题 3"), Some(3));
        assert_eq!(heading_level("Title"), Some(1));
        assert_eq!(heading_level("Normal"), None);
        assert_eq!(heading_level(""), None);
    }

    #[test]
    fn parses_paragraphs_and_style() {
        let xml = r#"<?xml version="1.0"?>
        <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
          <w:body>
            <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>第一章</w:t></w:r></w:p>
            <w:p><w:r><w:t>正文</w:t></w:r><w:r><w:t>内容</w:t></w:r></w:p>
          </w:body>
        </w:document>"#;

        let paragraphs = parse_paragraphs(xml).expect("应能解析");
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0].0, "Heading1");
        assert_eq!(paragraphs[0].1, "第一章");
        assert_eq!(paragraphs[1].1, "正文内容");
    }
}
