import { Channel, invoke } from "@tauri-apps/api/core";

import type {
  AppInfo,
  ChatEvent,
  Conversation,
  ConversationDetail,
  ConversationSummary,
  DocumentSummary,
  IngestJob,
  Library,
  LibrarySummary,
} from "@/types";

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

/** 列出知识库下的会话。 */
export async function listConversations(libraryId: string): Promise<ConversationSummary[]> {
  return invoke<ConversationSummary[]>("conversation_list", { libraryId });
}

/** 创建会话。 */
export async function createConversation(libraryId: string, title?: string): Promise<Conversation> {
  return invoke<Conversation>("conversation_create", { libraryId, title: title ?? null });
}

/** 读取会话详情。 */
export async function getConversation(id: string): Promise<ConversationDetail> {
  return invoke<ConversationDetail>("conversation_get", { id });
}

/** 重命名会话。 */
export async function renameConversation(id: string, title: string): Promise<Conversation> {
  return invoke<Conversation>("conversation_rename", { id, title });
}

/** 删除会话。 */
export async function deleteConversation(id: string): Promise<void> {
  await invoke<void>("conversation_delete", { id });
}

/** 打开应用数据目录。 */
export async function openDataDir(): Promise<void> {
  await invoke<void>("open_data_dir");
}

/** 流式问答：通过 Tauri Channel 接收事件。 */
export async function streamChat(
  libraryId: string,
  question: string,
  conversationId: string | null,
  history: Array<{ role: string; content: string }>,
  onEvent: (event: ChatEvent) => void,
): Promise<void> {
  const channel = new Channel<ChatEvent>();
  channel.onmessage = onEvent;
  await invoke("chat_stream", {
    libraryId,
    question,
    conversationId,
    history,
    channel,
  });
}

/** 非流式问答（用于自动化验收与快速调试）。 */
export async function completeChat(
  libraryId: string,
  question: string,
  conversationId: string | null = null,
): Promise<{
  provider: string;
  answer: string;
  citations: import("@/types").Citation[];
  conversationId: string;
}> {
  return invoke("chat_complete", { libraryId, question, conversationId });
}

/** 查询 DeepSeek API Key 状态。 */
export async function apiKeyStatus(): Promise<import("@/types").ApiKeyStatus> {
  return invoke("api_key_status");
}

/** 保存 DeepSeek API Key 到系统凭据库。 */
export async function apiKeySet(key: string): Promise<import("@/types").ApiKeyStatus> {
  return invoke("api_key_set", { key });
}

/** 删除系统凭据库中的 Key。 */
export async function apiKeyClear(): Promise<import("@/types").ApiKeyStatus> {
  return invoke("api_key_clear");
}

/** 探测三个 Provider 的可用状态。 */
export async function providersDetect(): Promise<import("@/types").ProviderDetection> {
  return invoke("providers_detect");
}

/** 读取设置项。 */
export async function settingsGet(key: string): Promise<string | null> {
  return invoke("settings_get", { key });
}

/** 写入设置项。 */
export async function settingsSet(key: string, value: string): Promise<void> {
  await invoke<void>("settings_set", { key, value });
}

/** 查询本地嵌入模型状态。 */
export async function embeddingStatus(): Promise<import("@/types").EmbeddingStatus> {
  return invoke("embedding_status");
}

/** 手动补写一批向量。 */
export async function embeddingBackfill(): Promise<number> {
  return invoke("embedding_backfill");
}
