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
export type AppView = "library" | "chat" | "settings";

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

/** 深浅色主题模式。 */
export type ThemeMode = "system" | "light" | "dark";
