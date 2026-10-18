import { create } from "zustand";

/** 支持的语言。 */
export type Language = "zh" | "en";

const STORAGE_KEY = "lumos.language";

/** 英文词典：以中文原文为键；`{name}` 为占位符。 */
const ENGLISH: Record<string, string> = {
  知识库: "Libraries",
  问答: "Chat",
  设置: "Settings",
  本地优先的引用式知识库: "Local-first knowledge base with citations",
  主导航: "Main navigation",
  "主题：{mode}": "Theme: {mode}",
  "主题：{mode}（点击切换）": "Theme: {mode} (click to switch)",
  "当前主题：{mode}": "Current theme: {mode}",
  跟随系统: "System",
  浅色: "Light",
  深色: "Dark",
  所有数据只保存在本机: "All data stays on this device",
  本地模式: "Local mode",
  "正在读取数据目录…": "Reading data directory…",
  关闭: "Close",
  取消: "Cancel",
  保存: "Save",
  删除: "Delete",
  创建: "Create",
  "处理中…": "Working…",
  知道了: "OK",
  "操作失败，请重试": "Something went wrong. Please try again.",

  "每个知识库独立索引；所有内容只保存在这台电脑上。":
    "Each library has its own index. Everything stays on this computer.",
  新建知识库: "New library",
  还没有知识库: "No libraries yet",
  "创建一个知识库，然后把 PDF、EPUB、Word、Markdown 或网页拖进来。索引、向量与问答记录全部保存在本机。":
    "Create a library and add PDF, EPUB, Word, Markdown or web pages. Indexes, vectors and chats stay on this machine.",
  创建第一个知识库: "Create your first library",
  知识库名称: "Library name",
  "例如：合同条款、毕业论文、读书笔记": "e.g. Contracts, Thesis, Reading notes",
  "知识库用于隔离不同主题的文档与索引。":
    "Libraries keep documents and indexes for different topics separate.",
  请输入知识库名称: "Please enter a library name",
  "名称不能超过 {max} 个字符": "Name must be at most {max} characters",
  重命名知识库: "Rename library",
  "删除知识库？": "Delete library?",
  "「{name}」里的文档、索引与问答记录都会从本机删除，且无法恢复。":
    'All documents, indexes and chats in "{name}" will be deleted from this device and cannot be recovered.',
  文档: "Docs",
  分块: "Chunks",
  占用: "Size",
  "更新于 {time}": "Updated {time}",
  "重命名 {name}": "Rename {name}",
  "删除 {name}": "Delete {name}",
  知识库列表: "Library list",
  返回知识库列表: "Back to libraries",
  刷新: "Refresh",
  暂停: "Pause",
  继续导入: "Resume import",
  导入文件: "Import files",
  导入文件夹: "Import folder",
  去提问: "Ask questions",
  还没有文档: "No documents yet",
  "导入 PDF、EPUB、Word、Markdown、网页或代码文件。解析与分块都在本机完成，源文件不会被移动或上传。":
    "Import PDF, EPUB, Word, Markdown, web pages or code files. Parsing and chunking happen locally; source files are never moved or uploaded.",
  "{docs} 篇文档 · {chunks} 个分块 · 更新于 {time}":
    "{docs} documents · {chunks} chunks · updated {time}",
  "{count} 个分块": "{count} chunks",
  "{pages} 页": "{pages} pages",
  排队中: "Queued",
  解析中: "Parsing",
  已就绪: "Ready",
  失败: "Failed",
  "正在解析 {name}": "Parsing {name}",
  "队列中还有 {count} 个文件": "{count} files still in queue",
  等待导入: "Waiting to import",
  导入已暂停: "Import paused",
  恢复后会继续处理队列中的文件: "Resume to continue processing the queue",
  取消任务: "Cancel task",
  "最近一次导入失败：{message}（{file}）": "Last import failed: {message} ({file})",
  关闭提示: "Dismiss",
  "删除中…": "Deleting…",
  选择要导入的文件夹: "Choose a folder to import",
  选择要导入的文档: "Choose documents to import",
  文档列表: "Document list",
  未知原因: "Unknown reason",
  关闭错误提示: "Dismiss error",
  刷新列表: "Refresh list",
  "已加入 {count} 个导入任务": "Added {count} import tasks",
  没有新的可导入文件: "No new files to import",
  "浏览器预览模式：请在桌面应用里导入文档": "Browser preview: import documents in the desktop app",
  "浏览器预览模式：无法读取本地知识库": "Browser preview: cannot read local libraries",
  "浏览器预览模式：无法读取本地文档": "Browser preview: cannot read local documents",
  "浏览器预览模式：未连接 Tauri 运行时": "Browser preview: not connected to the Tauri runtime",
  "删除文档 {title}": "Delete document {title}",

  对话: "Conversations",
  新建对话: "New chat",
  "还没有对话，直接在右侧提问即可开始。": "No chats yet — just ask on the right to start.",
  重命名对话: "Rename chat",
  "删除对话？": "Delete chat?",
  "「{title}」的所有消息都会从本机删除，且无法恢复。":
    'All messages in "{title}" will be deleted from this device and cannot be recovered.',
  标题: "Title",
  "重命名对话 {title}": "Rename chat {title}",
  "删除对话 {title}": "Delete chat {title}",
  选择知识库: "Select library",
  "{docs} 篇文档 · {chunks} 个分块": "{docs} documents · {chunks} chunks",
  隐藏引用面板: "Hide citation panel",
  显示引用面板: "Show citation panel",
  "基于「{name}」提问": 'Ask about "{name}"',
  "回答会标注引用编号，点击即可查看原文片段。":
    "Answers include citation numbers; click one to see the source passage.",
  "试试这样问：": "Try asking:",
  "有哪些工作用了对比学习做医学图像分割？":
    "Which papers use contrastive learning for medical image segmentation?",
  "合同里对违约金上限是怎么约定的？": "What does the contract say about the penalty cap?",
  "我去年夏天为什么决定换工作？": "Why did I decide to change jobs last summer?",
  发送: "Send",
  发送问题: "Send question",
  生成中: "Generating",
  "基于你的文档提问，Enter 发送，Shift+Enter 换行":
    "Ask about your documents. Enter to send, Shift+Enter for a new line.",
  先创建一个知识库并导入文档: "Create a library and import documents first",
  "回答只依据你导入的文档生成；没有引用依据时会明确说明。":
    "Answers are grounded in your documents and say so when no evidence is found.",
  "正在检索并生成回答…": "Searching and generating…",
  引用来源: "Sources",
  引用面板: "Citation panel",
  "回答中的 [1]、[2] 会在这里显示原文片段。":
    "Citations like [1], [2] appear here with the source passage.",
  "点击任意编号即可跳转对照。": "Click any number to jump to the passage.",
  "查看引用 {index}": "View citation {index}",
  关闭引用面板: "Close citation panel",
  "第 {page} 页": "Page {page}",
  先创建一个知识库: "Create a library first",
  "问答基于你导入的文档。先创建知识库并导入 PDF、Word、Markdown 等文件，再回来提问。":
    "Chat is grounded in your documents. Create a library, import files, then come back to ask.",
  去创建知识库: "Create a library",
  我: "Me",
  提问输入框: "Question input",
  离线关键词模式: "Offline keyword mode",
  "演示模式 · Demo": "Demo mode",

  "所有配置只保存在本机；API Key 存在系统凭据库，不会写入数据库或日志。":
    "All settings stay on this device. API keys live in the OS credential store, never in the database or logs.",
  "DeepSeek API Key": "DeepSeek API key",
  "当前状态：{status}": "Status: {status}",
  " 已配置（系统凭据库）": "configured (OS credential store)",
  " 已配置（环境变量）": "configured (environment variable)",
  " 未配置，将使用本地 Ollama 或离线关键词模式":
    "not configured; will use local Ollama or offline keyword mode",
  保存到凭据库: "Save to credential store",
  模型服务: "Model providers",
  当前使用: "In use",
  "Ollama 地址": "Ollama URL",
  "Ollama 模型": "Ollama model",
  "DeepSeek 模型": "DeepSeek model",
  保存设置: "Save settings",
  重新检测: "Detect again",
  本地嵌入模型: "Local embedding model",
  "已就绪：{model}（{dim} 维）": "Ready: {model} ({dim} dims)",
  "正在加载/下载模型（首次约 95MB）…": "Loading/downloading the model (about 95MB on first run)…",
  "不可用：{message}，检索将降级为关键词模式":
    "Unavailable: {message}. Retrieval falls back to keyword mode.",
  "读取中…": "Loading…",
  立即补写向量: "Backfill embeddings now",
  刷新状态: "Refresh status",
  外观: "Appearance",
  界面语言: "Language",
  中文: "Chinese",
  English: "English",
  数据目录: "Data directory",
  "数据库、日志与嵌入模型都保存在这里；卸载前可手动备份整个目录。":
    "Database, logs and embedding model live here. You can back up this folder manually before uninstalling.",
  打开数据目录: "Open data directory",
  数据导出: "Export",
  "把全部知识库、文档分块与对话导出为 Markdown + JSON，导出后不依赖 Lumos 也能阅读。":
    "Export all libraries, document chunks and chats as Markdown + JSON. The result is readable without Lumos.",
  导出全部数据: "Export all data",
  选择导出目录: "Choose export folder",
  "已导出 {libraries} 个知识库、{documents} 篇文档、{conversations} 个会话到 {root}":
    "Exported {libraries} libraries, {documents} documents and {conversations} chats to {root}",
  "已补写 {count} 个分块的向量": "Backfilled embeddings for {count} chunks",
  "API Key 已保存到系统凭据库": "API key saved to the OS credential store",
  "已删除系统凭据库中的 API Key": "Removed the API key from the OS credential store",
  设置已保存: "Settings saved",
  "请输入 API Key": "Please enter an API key",

  界面遇到了一点问题: "Something went wrong",
  "数据没有丢失。可以先尝试重新加载界面；如果反复出现，请在数据目录的 logs 中查看日志并反馈。":
    "Your data is safe. Try reloading the interface; if it keeps happening, check the logs in the data directory and report it.",
  重新加载: "Reload",
  未知错误: "Unknown error",
};

interface I18nState {
  /** 当前语言。 */
  language: Language;
  /** 切换语言。 */
  setLanguage: (language: Language) => void;
}

function readLanguage(): Language {
  if (typeof window === "undefined") return "zh";
  try {
    return window.localStorage.getItem(STORAGE_KEY) === "en" ? "en" : "zh";
  } catch {
    return "zh";
  }
}

/** 语言状态（持久化到 localStorage）。 */
export const useI18nStore = create<I18nState>((set) => ({
  language: readLanguage(),
  setLanguage: (language) => {
    try {
      window.localStorage.setItem(STORAGE_KEY, language);
    } catch {
      // 忽略持久化失败
    }
    set({ language });
  },
}));

/**
 * 翻译一段中文原文。
 *
 * @param language 目标语言
 * @param source 中文原文（同时作为 key）
 * @param params 占位符参数
 */
export function translate(
  language: Language,
  source: string,
  params?: Record<string, string | number>,
): string {
  const template = language === "en" ? (ENGLISH[source] ?? source) : source;
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (_match, key: string) =>
    params[key] === undefined ? `{${key}}` : String(params[key]),
  );
}

/** 组件内使用的翻译 hook。 */
export function useT(): (source: string, params?: Record<string, string | number>) => string {
  const language = useI18nStore((state) => state.language);
  return (source, params) => translate(language, source, params);
}
