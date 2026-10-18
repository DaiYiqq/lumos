//! 单文件摄取流水线：哈希 → 解析 → 分块 → 原子入库。

use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::db::Database;
use crate::db::models::DocumentStatus;
use crate::db::repo::documents::{self, NewChunk, NewDocument};
use crate::error::{AppError, AppResult};
use crate::ingest::chunk::{Chunk, DEFAULT_CHUNK_OVERLAP, DEFAULT_CHUNK_SIZE, chunk_sections};
use crate::ingest::extract;

/// 单个文件的摄取结果。
pub struct IngestOutcome {
    /// 写入的分块数。
    pub chunk_count: usize,
    /// 是否因内容重复而被跳过。
    pub skipped: bool,
    /// 结果说明（跳过原因等）。
    pub message: Option<String>,
}

/// 摄取一个文件。
///
/// # Errors
///
/// 文件不可读、格式不支持、解析失败、内容为空或用户取消时返回错误。
/// 取消使用 [`AppError::Cancelled`]，由队列转成 cancelled 状态。
pub fn ingest_file(
    database: &Database,
    library_id: &str,
    path: &Path,
    is_cancelled: impl Fn() -> bool,
) -> AppResult<IngestOutcome> {
    if is_cancelled() {
        return Err(AppError::Cancelled);
    }
    if !extract::supports(path) {
        return Err(AppError::InvalidInput(
            "暂不支持这种文件格式，已跳过".to_string(),
        ));
    }

    let metadata = std::fs::metadata(path)?;
    let content_hash = content_hash(path)?;

    let existing = database.with_connection(|connection| {
        documents::find_ready_by_hash(connection, library_id, &content_hash)
    })?;
    if existing.is_some() {
        return Ok(IngestOutcome {
            chunk_count: 0,
            skipped: true,
            message: Some("内容已存在，跳过重复索引".to_string()),
        });
    }

    let extracted = extract::extract(path)?;
    if is_cancelled() {
        return Err(AppError::Cancelled);
    }

    let chunks = chunk_sections(
        &extracted.sections,
        DEFAULT_CHUNK_SIZE,
        DEFAULT_CHUNK_OVERLAP,
    );
    if chunks.is_empty() {
        return Err(AppError::InvalidInput(
            "没有提取到可索引的正文，已跳过".to_string(),
        ));
    }
    if is_cancelled() {
        return Err(AppError::Cancelled);
    }

    let chunk_count = chunks.len();
    let new_chunks: Vec<NewChunk> = chunks.into_iter().map(to_new_chunk).collect();
    let path_text = path.to_string_lossy().to_string();
    let document = NewDocument {
        library_id,
        path: &path_text,
        title: &extracted.title,
        mime: &extracted.mime,
        size: i64::try_from(metadata.len()).unwrap_or(i64::MAX),
        page_count: extracted.page_count.map(i64::from),
        content_hash: &content_hash,
    };

    database.with_connection(|connection| {
        documents::upsert_document(
            connection,
            &document,
            &new_chunks,
            DocumentStatus::Ready,
            None,
        )
    })?;

    Ok(IngestOutcome {
        chunk_count,
        skipped: false,
        message: None,
    })
}

/// 把分块转成待写入结构。
fn to_new_chunk(chunk: Chunk) -> NewChunk {
    NewChunk {
        content: chunk.content,
        page_no: chunk.page_no,
        section_path: chunk.section_path,
        char_start: chunk.char_start,
        char_end: chunk.char_end,
        token_count: chunk.token_count,
    }
}

/// 流式计算文件 SHA-256，避免把大文件整体读进内存。
fn content_hash(path: &Path) -> AppResult<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repo::{documents, libraries};
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    /// 创建本次测试专用的临时目录。
    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lumos-test-{prefix}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("临时目录应可创建");
        dir
    }

    /// 用最小内容生成一个 zip 文件（DOCX / EPUB 都是 zip 容器）。
    fn write_zip(path: &Path, entries: &[(&str, &str)]) {
        let file = std::fs::File::create(path).expect("应能创建 zip");
        let mut archive = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        for (name, content) in entries {
            archive
                .start_file(*name, options)
                .expect("应能写入 zip 条目");
            archive
                .write_all(content.as_bytes())
                .expect("应能写入 zip 内容");
        }
        archive.finish().expect("zip 应收尾");
    }

    /// 建库 + 迁移，返回（数据库, 知识库 ID）。
    fn setup_database() -> (Database, String) {
        let database = Database::open_in_memory().expect("内存库应可创建");
        database.migrate().expect("迁移应成功");
        let library = database
            .with_connection(|connection| libraries::create_library(connection, "测试库"))
            .expect("建库应成功");
        (database, library.id)
    }

    #[test]
    fn ingests_hundred_documents_and_reports_statistics() {
        let (database, library_id) = setup_database();
        let dir = temp_dir("bulk");

        let mut files = Vec::new();
        for index in 0..100 {
            let path = dir.join(format!("note-{index:03}.md"));
            let content = format!(
                "# 文档 {index}\n\n## 背景\n\n这是第 {index} 篇用于压力测试的文档。\n\n## 正文\n\n{}\n\n## 结论\n\n分块应当覆盖标题层级。",
                "正文内容用于验证分块与入库。".repeat(30)
            );
            std::fs::write(&path, content).expect("应能写入 fixture");
            files.push(path);
        }

        for path in &files {
            let outcome = ingest_file(&database, &library_id, path, || false).expect("索引应成功");
            assert!(outcome.chunk_count > 0, "每个文件都应产生分块");
        }

        let (documents, summaries) = database
            .with_connection(|connection| {
                let documents = documents::list_documents(connection, &library_id)?;
                let summaries = libraries::list_libraries(connection)?;
                Ok((documents, summaries))
            })
            .expect("统计应成功");

        assert_eq!(documents.len(), files.len());
        assert!(documents.iter().all(|document| document.status == "ready"));
        assert_eq!(summaries[0].document_count, files.len() as i64);
        assert!(summaries[0].chunk_count >= files.len() as i64);

        // 增量索引：同一文件再次导入应被跳过
        let again =
            ingest_file(&database, &library_id, &files[0], || false).expect("重复索引应成功");
        assert!(again.skipped);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn ingests_docx_epub_and_html_fixtures() {
        let (database, library_id) = setup_database();
        let dir = temp_dir("formats");

        let docx_path = dir.join("contract.docx");
        write_zip(
            &docx_path,
            &[(
                "word/document.xml",
                r#"<?xml version="1.0" encoding="UTF-8"?>
                    <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                      <w:body>
                        <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>违约金条款</w:t></w:r></w:p>
                        <w:p><w:r><w:t>违约金上限为合同总额的百分之二十。</w:t></w:r></w:p>
                      </w:body>
                    </w:document>"#,
            )],
        );

        let epub_path = dir.join("book.epub");
        write_zip(
            &epub_path,
            &[
                ("mimetype", "application/epub+zip"),
                (
                    "META-INF/container.xml",
                    r#"<?xml version="1.0"?>
                    <container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
                      <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
                    </container>"#,
                ),
                (
                    "OEBPS/content.opf",
                    r#"<?xml version="1.0"?>
                    <package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/">
                      <metadata><dc:title>对比学习论文集</dc:title></metadata>
                      <manifest><item id="c1" href="chapter1.xhtml" media-type="application/xhtml+xml"/></manifest>
                      <spine><itemref idref="c1"/></spine>
                    </package>"#,
                ),
                (
                    "OEBPS/chapter1.xhtml",
                    r#"<html><head><title>第一章</title></head><body>
                    <h1>对比学习</h1><p>对比学习在医学图像分割中有广泛应用。</p></body></html>"#,
                ),
            ],
        );

        let html_path = dir.join("page.html");
        std::fs::write(
            &html_path,
            "<html><body><h1>网页笔记</h1><p>这是一段网页正文，用于验证 HTML 解析。</p></body></html>",
        )
        .expect("应能写入 HTML fixture");

        for path in [&docx_path, &epub_path, &html_path] {
            let outcome = ingest_file(&database, &library_id, path, || false).expect("索引应成功");
            assert!(outcome.chunk_count > 0, "{} 应产生分块", path.display());
        }

        let documents = database
            .with_connection(|connection| documents::list_documents(connection, &library_id))
            .expect("列表应成功");
        assert_eq!(documents.len(), 3);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn empty_file_returns_friendly_error() {
        let (database, library_id) = setup_database();
        let dir = temp_dir("empty");
        let path = dir.join("empty.txt");
        std::fs::write(&path, "   ").expect("应能写入空文件");

        let result = ingest_file(&database, &library_id, &path, || false);
        match result {
            Err(AppError::InvalidInput(message)) => {
                assert!(message.contains("空") || message.contains("正文"));
            }
            other => panic!(
                "应返回面向用户的错误，实际是 {other:?}",
                other = other.err()
            ),
        }

        std::fs::remove_dir_all(&dir).ok();
    }
}
