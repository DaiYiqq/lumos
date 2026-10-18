//! EPUB 文本提取。
//!
//! 直接解析 EPUB 容器：`META-INF/container.xml` → OPF → spine 顺序 →
//! 每个 XHTML 章节用 html2text 转成 Markdown 后按标题切章节。
//! OPF 解析失败时回退到「按文件名顺序读取所有 xhtml/html」。

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::error::{AppError, AppResult};
use crate::ingest::chunk::Section;
use crate::ingest::extract::{ExtractedDocument, file_title, split_markdown};

/// 提取 EPUB 正文。
///
/// # Errors
///
/// 容器损坏、没有正文或章节无法解码时返回面向用户的错误。
pub fn extract(path: &Path, fallback_title: &str) -> AppResult<ExtractedDocument> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .map_err(|error| AppError::InvalidInput(format!("无法打开 EPUB：{error}")))?;

    let container = read_entry(&mut archive, "META-INF/container.xml")
        .map_err(|_| AppError::InvalidInput("这份 EPUB 缺少容器信息，可能已损坏".to_string()))?;
    let opf_path = find_rootfile(&container)
        .ok_or_else(|| AppError::InvalidInput("这份 EPUB 缺少 OPF 目录文件".to_string()))?;
    let opf = read_entry(&mut archive, &opf_path)
        .map_err(|_| AppError::InvalidInput("无法读取 EPUB 的 OPF 目录文件".to_string()))?;

    let title = opf_title(&opf).unwrap_or_else(|| {
        if fallback_title.trim().is_empty() {
            file_title(path)
        } else {
            fallback_title.to_string()
        }
    });

    let chapters = spine_hrefs(&opf);
    let opf_directory = opf_path
        .rsplit_once('/')
        .map(|(directory, _)| format!("{directory}/"))
        .unwrap_or_default();
    let mut sections: Vec<Section> = Vec::new();

    if chapters.is_empty() {
        let mut names: Vec<String> = archive
            .file_names()
            .filter_map(|name| name.ok())
            .map(|name| name.into_owned())
            .filter(|name| name.ends_with(".xhtml") || name.ends_with(".html"))
            .filter(|name| !name.to_ascii_lowercase().contains("nav"))
            .collect();
        names.sort();
        for name in names {
            append_chapter(&mut archive, &name, &mut sections);
        }
    } else {
        for href in chapters {
            let resolved = if href.starts_with('/') {
                href.trim_start_matches('/').to_string()
            } else {
                format!("{opf_directory}{href}")
            };
            append_chapter(&mut archive, &resolved, &mut sections);
        }
    }

    if sections.is_empty() {
        return Err(AppError::InvalidInput(
            "EPUB 里没有可提取的文字内容".to_string(),
        ));
    }

    Ok(ExtractedDocument {
        title,
        mime: "application/epub+zip".to_string(),
        page_count: None,
        sections,
    })
}

/// 读取 zip 内的一个条目为字符串。
fn read_entry(archive: &mut zip::ZipArchive<BufReader<File>>, name: &str) -> AppResult<String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| AppError::InvalidInput(format!("EPUB 缺少条目：{name}")))?;
    let mut content = String::new();
    entry.read_to_string(&mut content)?;
    Ok(content)
}

/// 从 container.xml 找到 OPF 路径。
fn find_rootfile(container: &str) -> Option<String> {
    let mut reader = Reader::from_str(container);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) | Ok(Event::Empty(event)) => {
                if event.local_name().as_ref() == "rootfile"
                    && let Some(value) = attribute(&event, "full-path")
                {
                    return Some(value);
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
        buffer.clear();
    }
}

/// 从 OPF 中读取 dc:title。
fn opf_title(opf: &str) -> Option<String> {
    let mut reader = Reader::from_str(opf);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut in_title = false;
    let mut title = String::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) | Ok(Event::Empty(event)) => {
                if event.local_name().as_ref() == "title" {
                    in_title = true;
                }
            }
            Ok(Event::Text(event)) => {
                if in_title {
                    title.push_str(&crate::ingest::extract::text_content(&event));
                }
            }
            Ok(Event::End(event)) => {
                if event.local_name().as_ref() == "title" {
                    in_title = false;
                    let trimmed = title.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
        buffer.clear();
    }
}

/// 按 spine 顺序返回章节文件路径（相对压缩包根目录）。
fn spine_hrefs(opf: &str) -> Vec<String> {
    let mut manifest: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut spine: Vec<String> = Vec::new();

    let mut reader = Reader::from_str(opf);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) | Ok(Event::Empty(event)) => {
                match event.local_name().as_ref() {
                    "item" => {
                        let id = attribute(&event, "id");
                        let href = attribute(&event, "href");
                        let media_type = attribute(&event, "media-type");
                        if let (Some(id), Some(href)) = (id, href) {
                            let is_html = media_type
                                .map(|value| value.contains("html"))
                                .unwrap_or(true);
                            if is_html {
                                manifest.insert(id, href);
                            }
                        }
                    }
                    "itemref" => {
                        if let Some(value) = attribute(&event, "idref") {
                            spine.push(value);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
        buffer.clear();
    }

    spine
        .into_iter()
        .filter_map(|id| manifest.get(&id).cloned())
        .collect()
}

/// 读取一个章节并追加到 sections。
fn append_chapter(
    archive: &mut zip::ZipArchive<BufReader<File>>,
    name: &str,
    sections: &mut Vec<Section>,
) {
    let Ok(mut entry) = archive.by_name(name) else {
        return;
    };
    let mut bytes = Vec::new();
    if entry.read_to_end(&mut bytes).is_err() {
        return;
    }
    let Ok(markdown) = html2text::from_read(bytes.as_slice(), 100) else {
        return;
    };
    sections.extend(split_markdown(&markdown));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_rootfile_in_container() {
        let container = r#"<?xml version="1.0"?>
        <container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
          <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
        </container>"#;
        assert_eq!(
            find_rootfile(container).as_deref(),
            Some("OEBPS/content.opf")
        );
    }

    #[test]
    fn parses_spine_order_and_title() {
        let opf = r#"<?xml version="1.0"?>
        <package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/">
          <metadata><dc:title>示例书</dc:title></metadata>
          <manifest>
            <item id="c1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
            <item id="c2" href="chapter2.xhtml" media-type="application/xhtml+xml"/>
          </manifest>
          <spine><itemref idref="c2"/><itemref idref="c1"/></spine>
        </package>"#;
        assert_eq!(opf_title(opf).as_deref(), Some("示例书"));
        assert_eq!(spine_hrefs(opf), vec!["chapter2.xhtml", "chapter1.xhtml"]);
    }
}
