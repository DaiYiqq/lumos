//! 一键导出：把全部知识库、文档分块与对话导出为 Markdown + JSON。

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;
use tauri::State;

use crate::db::Database;
use crate::db::repo::{chunks, conversations as conversation_repo, documents, libraries};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// 导出结果摘要。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSummary {
    /// 导出根目录。
    pub root: String,
    /// 知识库数量。
    pub libraries: usize,
    /// 文档数量。
    pub documents: usize,
    /// 会话数量。
    pub conversations: usize,
}

/// 导出全部数据到指定目录。
///
/// # Errors
///
/// 目录不可写或数据库读取失败时返回错误。
#[tauri::command]
pub fn export_all(state: State<'_, AppState>, target_dir: String) -> Result<ExportSummary, String> {
    export(&state.db, Path::new(&target_dir)).map_err(|error| error.user_message())
}

/// 执行导出。
///
/// # Errors
///
/// 目录不可写或数据库读取失败时返回错误。
pub fn export(database: &Database, target: &Path) -> AppResult<ExportSummary> {
    let root = target.join(format!("lumos-export-{}", timestamp()));
    std::fs::create_dir_all(&root)?;

    let library_list = database.with_connection(libraries::list_libraries)?;
    let mut document_metadata = Vec::new();
    let mut document_count = 0usize;
    let mut conversation_count = 0usize;

    for library in &library_list {
        let library_dir = root.join("libraries").join(sanitize(&library.library.name));
        std::fs::create_dir_all(&library_dir)?;

        let document_list = database.with_connection(|connection| {
            documents::list_documents(connection, &library.library.id)
        })?;

        for document in &document_list {
            let chunk_list = database.with_connection(|connection| {
                chunks::list_chunks_for_document(connection, &document.id)
            })?;

            let mut markdown = format!(
                "# {}\n\n> 来源：{}\n> 类型：{}\n> 大小：{} 字节\n\n",
                document.title, document.path, document.mime, document.size
            );
            for chunk in &chunk_list {
                if let Some(section) = &chunk.section_path
                    && !section.trim().is_empty()
                {
                    markdown.push_str(&format!("## {section}\n\n"));
                }
                markdown.push_str(chunk.content.trim());
                markdown.push_str("\n\n---\n\n");
            }

            let file = library_dir.join(format!("{}.md", sanitize(&document.title)));
            std::fs::write(file, markdown)?;
            document_count += 1;
            document_metadata.push(json!({
                "libraryId": library.library.id,
                "libraryName": library.library.name,
                "title": document.title,
                "path": document.path,
                "mime": document.mime,
                "size": document.size,
                "chunkCount": document.chunk_count,
            }));
        }

        let conversation_list = database.with_connection(|connection| {
            conversation_repo::list_conversations(connection, &library.library.id)
        })?;
        if !conversation_list.is_empty() {
            let conversation_dir = root
                .join("conversations")
                .join(sanitize(&library.library.name));
            std::fs::create_dir_all(&conversation_dir)?;

            for conversation in &conversation_list {
                let messages = database.with_connection(|connection| {
                    conversation_repo::list_messages(connection, &conversation.conversation.id)
                })?;
                let mut markdown = format!(
                    "# {}\n\n> 知识库：{}\n> 创建时间：{}\n\n",
                    conversation.conversation.title,
                    library.library.name,
                    conversation.conversation.created_at
                );
                for message in &messages {
                    let speaker = if message.role == "user" {
                        "用户"
                    } else {
                        "Lumos"
                    };
                    markdown.push_str(&format!("### {speaker}\n\n{}\n\n", message.content.trim()));
                }
                let file = conversation_dir
                    .join(format!("{}.md", sanitize(&conversation.conversation.title)));
                std::fs::write(file, markdown)?;
                conversation_count += 1;
            }
        }
    }

    let metadata = json!({
        "exportedAt": timestamp(),
        "libraries": library_list,
        "documents": document_metadata,
    });
    std::fs::write(
        root.join("libraries.json"),
        serde_json::to_string_pretty(&metadata)
            .map_err(|error| AppError::Internal(format!("导出元数据序列化失败：{error}")))?,
    )?;

    Ok(ExportSummary {
        root: root.display().to_string(),
        libraries: library_list.len(),
        documents: document_count,
        conversations: conversation_count,
    })
}

/// 当前时间戳（秒），用于导出目录名。
fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

/// 把标题转成安全文件名：保留中英文、数字与常用符号。
fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|character| {
            if character.is_alphanumeric()
                || matches!(character, ' ' | '-' | '_' | '.' | '(' | ')' | '（' | '）')
            {
                character
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.');
    let limited: String = trimmed.chars().take(60).collect();
    if limited.trim().is_empty() {
        "untitled".to_string()
    } else {
        limited
    }
}

/// 兼容未来扩展的导出子目录路径。
#[allow(dead_code)]
fn export_root(target: &Path) -> PathBuf {
    target.join("lumos-export")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::DocumentStatus;
    use crate::db::repo::documents::{NewChunk, NewDocument};
    use crate::db::repo::libraries::create_library;

    #[test]
    fn exports_markdown_and_metadata() {
        let database = Database::open_in_memory().expect("内存库应可创建");
        database.migrate().expect("迁移应成功");
        let library = database
            .with_connection(|connection| create_library(connection, "合同库"))
            .expect("建库应成功");

        database
            .with_connection(|connection| {
                documents::upsert_document(
                    connection,
                    &NewDocument {
                        library_id: &library.id,
                        path: "/tmp/contract.md",
                        title: "违约金条款",
                        mime: "text/markdown",
                        size: 128,
                        page_count: None,
                        content_hash: "hash",
                    },
                    &[NewChunk {
                        content: "违约金上限为合同总额的百分之二十。".to_string(),
                        page_no: None,
                        section_path: Some("第九条".to_string()),
                        char_start: 0,
                        char_end: 10,
                        token_count: 10,
                    }],
                    DocumentStatus::Ready,
                    None,
                )
            })
            .expect("写入文档应成功");

        let target = std::env::temp_dir().join(format!("lumos-export-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&target).expect("临时目录应可创建");
        let summary = export(&database, &target).expect("导出应成功");
        assert_eq!(summary.libraries, 1);
        assert_eq!(summary.documents, 1);

        let root = PathBuf::from(&summary.root);
        assert!(root.join("libraries.json").is_file());
        let files = std::fs::read_dir(root.join("libraries").join("合同库"))
            .expect("文档目录应存在")
            .count();
        assert_eq!(files, 1);

        std::fs::remove_dir_all(&target).ok();
    }
}
