import { invoke } from "@tauri-apps/api/core";

import type { AppInfo, DocumentSummary, IngestJob, Library, LibrarySummary } from "@/types";

/**
 * 判断当前是否运行在 Tauri 容器内。
 *
 * 单元测试与浏览器预览（`pnpm dev`）里没有 Tauri 注入的全局对象，
 * 业务代码据此决定是否调用 Rust 命令，避免在浏览器里抛错。
 */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** 读取应用元信息（版本、数据目录、平台）。 */
export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}

/** 列出全部知识库及统计信息。 */
export async function listLibraries(): Promise<LibrarySummary[]> {
  return invoke<LibrarySummary[]>("library_list");
}

/** 创建知识库。 */
export async function createLibrary(name: string): Promise<Library> {
  return invoke<Library>("library_create", { name });
}

/** 重命名知识库。 */
export async function renameLibrary(id: string, name: string): Promise<Library> {
  return invoke<Library>("library_rename", { id, name });
}

/** 删除知识库（级联删除其文档与分块）。 */
export async function deleteLibrary(id: string): Promise<void> {
  await invoke<void>("library_delete", { id });
}

/** 列出知识库下的文档。 */
export async function listDocuments(libraryId: string): Promise<DocumentSummary[]> {
  return invoke<DocumentSummary[]>("document_list", { libraryId });
}

/** 删除单个文档。 */
export async function deleteDocument(id: string): Promise<void> {
  await invoke<void>("document_delete", { id });
}

/** 列出知识库的导入任务。 */
export async function listIngestJobs(libraryId: string): Promise<IngestJob[]> {
  return invoke<IngestJob[]>("ingest_job_list", { libraryId });
}

/** 导入文件或文件夹，返回新建任务数量。 */
export async function importPaths(libraryId: string, paths: string[]): Promise<number> {
  return invoke<number>("ingest_import_paths", { libraryId, paths });
}

/** 暂停导入队列。 */
export async function pauseIngestQueue(): Promise<boolean> {
  return invoke<boolean>("ingest_pause");
}

/** 恢复导入队列。 */
export async function resumeIngestQueue(): Promise<boolean> {
  return invoke<boolean>("ingest_resume");
}

/** 查询队列是否暂停。 */
export async function getIngestQueueState(): Promise<boolean> {
  return invoke<boolean>("ingest_queue_state");
}

/** 取消一个导入任务。 */
export async function cancelIngestJob(jobId: string): Promise<void> {
  await invoke<void>("ingest_cancel_job", { jobId });
}
