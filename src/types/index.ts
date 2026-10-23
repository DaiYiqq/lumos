/** 全局共享的领域类型。 */

/** Rust 侧 `app_info` 命令返回的应用元信息。 */
export interface AppInfo {
  /** 产品名，例如 `Lumos`。 */
  name: string;
  /** 语义化版本号，例如 `0.1.0`。 */
  version: string;
  /** 应用数据目录的绝对路径（数据库、日志、设置都存放于此）。 */
  dataDir: string;
  /** 数据目录的展示形式：家目录前缀替换为 `~`，避免暴露用户名。 */
  dataDirDisplay: string;
  /** 运行平台标识，例如 `windows` / `macos` / `linux`。 */
  platform: string;
}

/** 侧边栏一级导航。 */
export type AppView = "library" | "search" | "chat" | "settings";

/** 检索模式。 */
export type SearchMode = "hybrid" | "vector" | "keyword";

/** 单条检索命中（与 Rust `SearchHit` 对应）。 */
export interface SearchHit {
  /** 分块 ID。 */
  chunkId: number;
  /** 文档 ID。 */
  documentId: string;
  /** 文档标题。 */
  documentTitle: string;
  /** 文档路径。 */
  documentPath: string;
  /** 页码。 */
  pageNo: number | null;
  /** 章节路径。 */
  sectionPath: string | null;
  /** 分块正文。 */
  content: string;
  /** 融合分数。 */
  score: number;
  /** 向量名次。 */
  vectorRank: number | null;
  /** 关键词名次。 */
  keywordRank: number | null;
}

/** 检索响应。 */
export interface SearchResponse {
  /** 原始查询。 */
  query: string;
  /** 请求的模式。 */
  requestedMode: string;
  /** 实际使用的模式。 */
  mode: string;
  /** 向量检索是否参与。 */
  vectorEnabled: boolean;
  /** 命中列表。 */
  hits: SearchHit[];
}

/** 知识库基础信息。 */
export interface Library {
  /** 知识库 UUID。 */
  id: string;
  /** 用户可见名称。 */
  name: string;
  /** 创建时间（RFC3339 UTC）。 */
  createdAt: string;
  /** 最后更新时间（RFC3339 UTC）。 */
  updatedAt: string;
  /** 该库使用的嵌入模型标识。 */
  embeddingModel: string;
  /** 分块配置（JSON 字符串）。 */
  chunkConfig: string;
}

/** 知识库列表项：基础信息 + 统计。 */
export interface LibrarySummary extends Library {
  /** 文档数量。 */
  documentCount: number;
  /** 分块数量。 */
  chunkCount: number;
  /** 文档体积总和（字节）。 */
  sizeBytes: number;
}

/** 文档处理状态。 */
export type DocumentStatus = "pending" | "indexing" | "ready" | "failed";

/** 文档列表项。 */
export interface DocumentSummary {
  /** 文档 UUID。 */
  id: string;
  /** 所属知识库。 */
  libraryId: string;
  /** 原始文件绝对路径。 */
  path: string;
  /** 展示标题。 */
  title: string;
  /** MIME 类型。 */
  mime: string;
  /** 文件大小（字节）。 */
  size: number;
  /** 页数（仅 PDF）。 */
  pageCount: number | null;
  /** 处理状态。 */
  status: DocumentStatus;
  /** 失败原因。 */
  errorMsg: string | null;
  /** 创建时间。 */
  createdAt: string;
  /** 最后更新时间。 */
  updatedAt: string;
  /** 分块数量。 */
  chunkCount: number;
}

/** 导入任务状态。 */
export type IngestJobStatus = "pending" | "running" | "done" | "failed" | "cancelled";

/** 导入任务。 */
export interface IngestJob {
  /** 任务 UUID。 */
  id: string;
  /** 所属知识库。 */
  libraryId: string;
  /** 文件路径。 */
  path: string;
  /** 任务状态。 */
  status: IngestJobStatus;
  /** 已写入分块数。 */
  chunkCount: number;
  /** 结果说明。 */
  message: string | null;
  /** 创建时间。 */
  createdAt: string;
  /** 最后更新时间。 */
  updatedAt: string;
}

/** 摄取进度事件（后端通过 `ingest://progress` 推送）。 */
export interface IngestProgress {
  /** 任务 ID。 */
  jobId: string;
  /** 知识库 ID。 */
  libraryId: string;
  /** 文件路径。 */
  path: string;
  /** 任务状态。 */
  status: IngestJobStatus;
  /** 已写入分块数。 */
  chunkCount: number;
  /** 结果说明。 */
  message: string | null;
}

/** 一次对话（会话）。 */
export interface Conversation {
  /** 会话 UUID。 */
  id: string;
  /** 所属知识库。 */
  libraryId: string;
  /** 标题。 */
  title: string;
  /** 创建时间。 */
  createdAt: string;
  /** 最后更新时间。 */
  updatedAt: string;
}

/** 会话列表项。 */
export interface ConversationSummary extends Conversation {
  /** 消息条数。 */
  messageCount: number;
}

/** 持久化的消息角色。 */
export type MessageRole = "user" | "assistant" | "system";

/** 持久化的消息。 */
export interface StoredMessage {
  /** 消息 UUID。 */
  id: string;
  /** 所属会话。 */
  conversationId: string;
  /** 角色。 */
  role: MessageRole;
  /** 正文。 */
  content: string;
  /** 引用 JSON（助手消息才有）。 */
  citationsJson: string | null;
  /** 创建时间。 */
  createdAt: string;
}

/** 会话详情。 */
export interface ConversationDetail {
  /** 会话信息。 */
  conversation: Conversation;
  /** 消息列表。 */
  messages: StoredMessage[];
}

/** 与回答一一对应的引用。 */
export interface Citation {
  /** 编号（与正文 `[n]` 对应）。 */
  index: number;
  /** 分块 ID。 */
  chunkId: number;
  /** 文档 ID。 */
  documentId: string;
  /** 文档标题。 */
  documentTitle: string;
  /** 文档路径。 */
  documentPath: string;
  /** 页码。 */
  pageNo: number | null;
  /** 章节路径。 */
  sectionPath: string | null;
  /** 原文片段。 */
  content: string;
  /** 融合检索分数。 */
  score: number;
}

/** 问答流式事件（与 Rust `ChatEvent` 对应）。 */
export type ChatEvent =
  | { type: "conversation"; id: string; title: string }
  | { type: "provider"; provider: string; label: string }
  | { type: "notice"; message: string }
  | { type: "citations"; citations: Citation[] }
  | { type: "delta"; text: string }
  | { type: "done"; provider: string }
  | { type: "error"; message: string };

/** API Key 状态。 */
export interface ApiKeyStatus {
  /** 是否已配置。 */
  configured: boolean;
  /** 来源：keyring / environment / none。 */
  source: string;
}

/** Provider 状态。 */
export interface ProviderStatus {
  /** deepseek / ollama / none。 */
  id: string;
  /** 是否可用。 */
  available: boolean;
  /** 是否会被自动选中。 */
  selected: boolean;
  /** 说明。 */
  detail: string;
}

/** Provider 探测结果。 */
export interface ProviderDetection {
  /** 三个 Provider 的状态。 */
  statuses: ProviderStatus[];
  /** 自动选择结果。 */
  selected: string;
}

/** 本地嵌入服务状态。 */
export interface EmbeddingStatus {
  /** loading / ready / failed。 */
  state: string;
  /** 模型标识。 */
  model: string;
  /** 维度。 */
  dimension: number;
  /** 失败原因。 */
  message: string | null;
}

/** 深浅色主题模式。 */
export type ThemeMode = "system" | "light" | "dark";
