/**
 * 在线 Demo 的模拟后端。
 *
 * 仅在 `vite build --mode demo` 时通过别名替换 `@/lib/tauri`，桌面版不会打包本文件。
 * 数据全部在浏览器内存中；刷新页面即重置，不会上传任何内容。
 */
import type {
  ApiKeyStatus,
  AppInfo,
  ChatEvent,
  Citation,
  Conversation,
  ConversationDetail,
  ConversationSummary,
  DocumentSummary,
  EmbeddingStatus,
  IngestJob,
  Library,
  LibrarySummary,
  ProviderDetection,
  StoredMessage,
} from "@/types";
import type { ExportSummary } from "@/types/export";

const NOW = "2026-10-18T08:00:00.000Z";

const LIBRARIES: Library[] = [
  {
    id: "demo-contracts",
    name: "合同条款",
    createdAt: NOW,
    updatedAt: NOW,
    embeddingModel: "BAAI/bge-small-zh-v1.5",
    chunkConfig: "{}",
  },
  {
    id: "demo-papers",
    name: "论文库",
    createdAt: NOW,
    updatedAt: NOW,
    embeddingModel: "BAAI/bge-small-zh-v1.5",
    chunkConfig: "{}",
  },
  {
    id: "demo-notes",
    name: "个人笔记",
    createdAt: NOW,
    updatedAt: NOW,
    embeddingModel: "BAAI/bge-small-zh-v1.5",
    chunkConfig: "{}",
  },
];

const DOCUMENTS: DocumentSummary[] = [
  {
    id: "demo-doc-contract",
    libraryId: "demo-contracts",
    path: "demo/contract.docx",
    title: "合同违约金条款",
    mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    size: 24576,
    pageCount: 3,
    status: "ready",
    errorMsg: null,
    createdAt: NOW,
    updatedAt: NOW,
    chunkCount: 2,
  },
  {
    id: "demo-doc-service",
    libraryId: "demo-contracts",
    path: "demo/service-agreement.md",
    title: "服务协议",
    mime: "text/markdown",
    size: 4096,
    pageCount: null,
    status: "ready",
    errorMsg: null,
    createdAt: NOW,
    updatedAt: NOW,
    chunkCount: 1,
  },
  {
    id: "demo-doc-contrastive",
    libraryId: "demo-papers",
    path: "demo/contrastive-learning.md",
    title: "对比学习在医学图像分割中的应用",
    mime: "text/markdown",
    size: 8192,
    pageCount: null,
    status: "ready",
    errorMsg: null,
    createdAt: NOW,
    updatedAt: NOW,
    chunkCount: 3,
  },
  {
    id: "demo-doc-survey",
    libraryId: "demo-papers",
    path: "demo/contrastive-survey.pdf",
    title: "Contrastive Learning Survey",
    mime: "application/pdf",
    size: 524288,
    pageCount: 12,
    status: "ready",
    errorMsg: null,
    createdAt: NOW,
    updatedAt: NOW,
    chunkCount: 1,
  },
  {
    id: "demo-doc-notes",
    libraryId: "demo-notes",
    path: "demo/personal-notes.md",
    title: "换工作的决定",
    mime: "text/markdown",
    size: 2048,
    pageCount: null,
    status: "ready",
    errorMsg: null,
    createdAt: NOW,
    updatedAt: NOW,
    chunkCount: 1,
  },
];

interface DemoChunk {
  id: number;
  documentId: string;
  title: string;
  path: string;
  pageNo: number | null;
  sectionPath: string | null;
  content: string;
}

const CHUNKS: DemoChunk[] = [
  {
    id: 201,
    documentId: "demo-doc-contract",
    title: "合同违约金条款",
    path: "demo/contract.docx",
    pageNo: 3,
    sectionPath: "第九条 违约金",
    content: "违约金上限为合同总额的百分之二十。",
  },
  {
    id: 202,
    documentId: "demo-doc-contract",
    title: "合同违约金条款",
    path: "demo/contract.docx",
    pageNo: 3,
    sectionPath: "第九条 违约金",
    content: "若实际损失超过该上限，守约方可另行主张赔偿。",
  },
  {
    id: 203,
    documentId: "demo-doc-service",
    title: "服务协议",
    path: "demo/service-agreement.md",
    pageNo: null,
    sectionPath: "服务期限",
    content: "本协议自双方签署之日起生效，有效期一年。",
  },
  {
    id: 101,
    documentId: "demo-doc-contrastive",
    title: "对比学习在医学图像分割中的应用",
    path: "demo/contrastive-learning.md",
    pageNo: null,
    sectionPath: "研究背景",
    content: "自监督对比学习用于医学图像分割，可减少对标注数据的依赖。",
  },
  {
    id: 102,
    documentId: "demo-doc-contrastive",
    title: "对比学习在医学图像分割中的应用",
    path: "demo/contrastive-learning.md",
    pageNo: null,
    sectionPath: "方法",
    content: "在编码器输出上做实例级对比损失，并结合分割头的像素级监督。",
  },
  {
    id: 103,
    documentId: "demo-doc-contrastive",
    title: "对比学习在医学图像分割中的应用",
    path: "demo/contrastive-learning.md",
    pageNo: null,
    sectionPath: "结果",
    content: "报告显示该工作使 Dice 提升 2.3 个点。",
  },
  {
    id: 104,
    documentId: "demo-doc-survey",
    title: "Contrastive Learning Survey",
    path: "demo/contrastive-survey.pdf",
    pageNo: 4,
    sectionPath: "3. Methods",
    content: "Contrastive pretraining learns representations by pulling positive pairs together.",
  },
  {
    id: 301,
    documentId: "demo-doc-notes",
    title: "换工作的决定",
    path: "demo/personal-notes.md",
    pageNo: null,
    sectionPath: "换工作的决定",
    content:
      "2025 年六月，我离开了工作五年的公司。真正的原因不是薪资，而是长期加班让我失去了写作的时间。",
  },
];

interface DemoConversation {
  conversation: Conversation;
  messages: StoredMessage[];
}

const conversations: DemoConversation[] = [];

function toCitation(chunk: DemoChunk, index: number): Citation {
  return {
    index,
    chunkId: chunk.id,
    documentId: chunk.documentId,
    documentTitle: chunk.title,
    documentPath: chunk.path,
    pageNo: chunk.pageNo,
    sectionPath: chunk.sectionPath,
    content: chunk.content,
    score: 0.032,
  };
}

function citationsFor(ids: number[]): Citation[] {
  return ids
    .map((id) => CHUNKS.find((chunk) => chunk.id === id))
    .filter((chunk): chunk is DemoChunk => Boolean(chunk))
    .map((chunk, index) => toCitation(chunk, index + 1));
}

interface DemoAnswer {
  markdown: string;
  citations: Citation[];
}

const DEMO_NOTICE = "在线演示：回答由浏览器内置示例文档生成，未调用真实模型。";

function buildAnswer(question: string, libraryId: string): DemoAnswer {
  if (/对比|医学|图像|contrastive/i.test(question)) {
    return {
      citations: citationsFor([101, 102, 103]),
      markdown: [
        "根据内置示例文档，相关工作总结如下：",
        "",
        "- 自监督对比学习被用于医学图像分割，以减少对标注数据的依赖 [1]。",
        "- 具体做法是在编码器输出上做实例级对比损失，并结合分割头的像素级监督 [2]。",
        "- 示例文档报告该工作使 Dice 提升 2.3 个点 [3]。",
        "",
        "> 在线 Demo 的回答由内置示例生成，未调用真实模型。",
      ].join("\n"),
    };
  }

  if (/违约金|合同|赔偿/i.test(question)) {
    return {
      citations: citationsFor([201, 202]),
      markdown: [
        "根据示例合同：",
        "",
        "- 违约金上限为合同总额的百分之二十 [1]。",
        "- 若实际损失超过该上限，守约方可另行主张赔偿 [2]。",
        "",
        "> 在线 Demo 的回答由内置示例生成，未调用真实模型。",
      ].join("\n"),
    };
  }

  if (/换工作|辞职|去年夏天|职业/i.test(question)) {
    return {
      citations: citationsFor([301]),
      markdown: [
        "根据示例笔记：",
        "",
        "- 你决定换工作的原因不是薪资，而是长期加班让你失去了写作的时间 [1]。",
        "- 笔记提到时间是 2025 年六月，离开工作了五年的公司 [1]。",
        "",
        "> 在线 Demo 的回答由内置示例生成，未调用真实模型。",
      ].join("\n"),
    };
  }

  // 默认：用当前知识库的第一个示例分块，并明确说明 Demo 的边界
  const fallback = CHUNKS.find((chunk) =>
    DOCUMENTS.some((doc) => doc.id === chunk.documentId && doc.libraryId === libraryId),
  );
  return {
    citations: fallback ? [toCitation(fallback, 1)] : [],
    markdown: [
      "在线 Demo 只内置了少量示例文档，没有找到与该问题直接相关的内容。",
      "",
      "可以试试这些问题：",
      "",
      "- 合同里对违约金上限是怎么约定的？",
      "- 有哪些工作用了对比学习做医学图像分割？",
      "- 我为什么决定换工作？",
      "",
      "> 下载桌面版后，可以导入自己的 PDF / Word / Markdown 并接入 DeepSeek 或 Ollama。",
    ].join("\n"),
  };
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

function summarize(library: Library): LibrarySummary {
  const documents = DOCUMENTS.filter((document) => document.libraryId === library.id);
  const chunkCount = documents.reduce((total, document) => total + document.chunkCount, 0);
  const sizeBytes = documents.reduce((total, document) => total + document.size, 0);
  return { ...library, documentCount: documents.length, chunkCount, sizeBytes };
}

function makeConversation(libraryId: string, title: string): DemoConversation {
  const conversation: Conversation = {
    id: `demo-conv-${Date.now()}`,
    libraryId,
    title: title.slice(0, 24) + (title.length > 24 ? "…" : ""),
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
  const entry: DemoConversation = { conversation, messages: [] };
  conversations.unshift(entry);
  return entry;
}

function persistMessage(
  entry: DemoConversation,
  role: "user" | "assistant",
  content: string,
  citations: Citation[] | null,
): void {
  entry.messages.push({
    id: `demo-msg-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
    conversationId: entry.conversation.id,
    role,
    content,
    citationsJson: citations ? JSON.stringify(citations) : null,
    createdAt: new Date().toISOString(),
  });
  entry.conversation.updatedAt = new Date().toISOString();
}

async function streamAnswer(
  libraryId: string,
  question: string,
  conversationId: string | null,
  onEvent: (event: ChatEvent) => void,
): Promise<void> {
  const existing = conversationId
    ? conversations.find((entry) => entry.conversation.id === conversationId)
    : undefined;
  const entry = existing ?? makeConversation(libraryId, question);
  const { markdown, citations } = buildAnswer(question, libraryId);

  onEvent({ type: "conversation", id: entry.conversation.id, title: entry.conversation.title });
  onEvent({ type: "provider", provider: "none", label: "演示模式 · Demo" });
  onEvent({ type: "notice", message: DEMO_NOTICE });
  onEvent({ type: "citations", citations });
  persistMessage(entry, "user", question, null);

  for (let index = 0; index < markdown.length; index += 4) {
    onEvent({ type: "delta", text: markdown.slice(index, index + 4) });
    await sleep(16);
  }
  persistMessage(entry, "assistant", markdown, citations);
  onEvent({ type: "done", provider: "none" });
}

export function isTauriRuntime(): boolean {
  return true;
}

export async function getAppInfo(): Promise<AppInfo> {
  return {
    name: "Lumos",
    version: "0.1.0-demo",
    dataDir: "Browser memory (online demo)",
    dataDirDisplay: "浏览器内存（在线演示）",
    platform: "web",
  };
}

export async function listLibraries(): Promise<LibrarySummary[]> {
  await sleep(120);
  return LIBRARIES.map(summarize);
}

export async function createLibrary(name: string): Promise<Library> {
  const library: Library = {
    id: `demo-lib-${Date.now()}`,
    name,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    embeddingModel: "BAAI/bge-small-zh-v1.5",
    chunkConfig: "{}",
  };
  LIBRARIES.push(library);
  return library;
}

export async function renameLibrary(id: string, name: string): Promise<Library> {
  const library = LIBRARIES.find((item) => item.id === id);
  if (!library) throw new Error("知识库不存在");
  library.name = name;
  library.updatedAt = new Date().toISOString();
  return library;
}

export async function deleteLibrary(id: string): Promise<void> {
  const index = LIBRARIES.findIndex((item) => item.id === id);
  if (index >= 0) LIBRARIES.splice(index, 1);
  for (let cursor = DOCUMENTS.length - 1; cursor >= 0; cursor -= 1) {
    if (DOCUMENTS[cursor]?.libraryId === id) DOCUMENTS.splice(cursor, 1);
  }
}

export async function listDocuments(libraryId: string): Promise<DocumentSummary[]> {
  await sleep(100);
  return DOCUMENTS.filter((document) => document.libraryId === libraryId);
}

export async function deleteDocument(id: string): Promise<void> {
  const index = DOCUMENTS.findIndex((document) => document.id === id);
  if (index >= 0) DOCUMENTS.splice(index, 1);
}

export async function listIngestJobs(_libraryId: string): Promise<IngestJob[]> {
  return [];
}

export async function importPaths(libraryId: string, paths: string[]): Promise<number> {
  for (const path of paths) {
    const title = path.split(/[\\/]/).pop() ?? "示例文档";
    DOCUMENTS.unshift({
      id: `demo-doc-${Date.now()}-${Math.random().toString(16).slice(2, 6)}`,
      libraryId,
      path,
      title,
      mime: "text/markdown",
      size: 1024,
      pageCount: null,
      status: "ready",
      errorMsg: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      chunkCount: 1,
    });
  }
  return paths.length;
}

export async function pauseIngestQueue(): Promise<boolean> {
  return true;
}

export async function resumeIngestQueue(): Promise<boolean> {
  return false;
}

export async function getIngestQueueState(): Promise<boolean> {
  return false;
}

export async function cancelIngestJob(_jobId: string): Promise<void> {
  // demo 没有真实任务
}

export async function listConversations(libraryId: string): Promise<ConversationSummary[]> {
  await sleep(80);
  return conversations
    .filter((entry) => entry.conversation.libraryId === libraryId)
    .map((entry) => ({
      ...entry.conversation,
      messageCount: entry.messages.length,
    }));
}

export async function createConversation(libraryId: string, title?: string): Promise<Conversation> {
  return makeConversation(libraryId, title ?? "新的对话").conversation;
}

export async function getConversation(id: string): Promise<ConversationDetail> {
  const entry = conversations.find((item) => item.conversation.id === id);
  if (!entry) throw new Error("会话不存在");
  return { conversation: entry.conversation, messages: entry.messages };
}

export async function renameConversation(id: string, title: string): Promise<Conversation> {
  const entry = conversations.find((item) => item.conversation.id === id);
  if (!entry) throw new Error("会话不存在");
  entry.conversation.title = title;
  entry.conversation.updatedAt = new Date().toISOString();
  return entry.conversation;
}

export async function deleteConversation(id: string): Promise<void> {
  const index = conversations.findIndex((item) => item.conversation.id === id);
  if (index >= 0) conversations.splice(index, 1);
}

export async function openDataDir(): Promise<void> {
  // 在线 Demo 没有本地目录
}

export async function streamChat(
  libraryId: string,
  question: string,
  conversationId: string | null,
  _history: Array<{ role: string; content: string }>,
  onEvent: (event: ChatEvent) => void,
): Promise<void> {
  await streamAnswer(libraryId, question, conversationId, onEvent);
}

export async function completeChat(
  libraryId: string,
  question: string,
  conversationId: string | null = null,
): Promise<{ provider: string; answer: string; citations: Citation[]; conversationId: string }> {
  const existing = conversationId
    ? conversations.find((entry) => entry.conversation.id === conversationId)
    : undefined;
  const entry = existing ?? makeConversation(libraryId, question);
  const { markdown, citations } = buildAnswer(question, libraryId);
  persistMessage(entry, "user", question, null);
  persistMessage(entry, "assistant", markdown, citations);
  return { provider: "none", answer: markdown, citations, conversationId: entry.conversation.id };
}

export async function apiKeyStatus(): Promise<ApiKeyStatus> {
  return { configured: true, source: "demo" };
}

export async function apiKeySet(_key: string): Promise<ApiKeyStatus> {
  return { configured: true, source: "demo" };
}

export async function apiKeyClear(): Promise<ApiKeyStatus> {
  return { configured: false, source: "none" };
}

export async function providersDetect(): Promise<ProviderDetection> {
  return {
    statuses: [
      {
        id: "deepseek",
        available: true,
        selected: true,
        detail: "在线演示：模拟 DeepSeek，无需 API Key",
      },
      { id: "ollama", available: false, selected: false, detail: "演示模式未连接 Ollama" },
      {
        id: "none",
        available: true,
        selected: false,
        detail: "离线关键词模式（演示中作为回退）",
      },
    ],
    selected: "deepseek",
  };
}

export async function settingsGet(key: string): Promise<string | null> {
  if (key === "ollama_base_url") return "http://127.0.0.1:11434";
  if (key === "deepseek_model") return "deepseek-chat";
  return null;
}

export async function settingsSet(_key: string, _value: string): Promise<void> {
  // 演示模式不持久化设置
}

export async function embeddingStatus(): Promise<EmbeddingStatus> {
  return {
    state: "ready",
    model: "BAAI/bge-small-zh-v1.5 (demo)",
    dimension: 512,
    message: null,
  };
}

export async function embeddingBackfill(): Promise<number> {
  return 0;
}

export async function exportAll(_targetDir: string): Promise<ExportSummary> {
  return {
    root: "在线演示：数据保留在浏览器内，未写入磁盘",
    libraries: LIBRARIES.length,
    documents: DOCUMENTS.length,
    conversations: conversations.length,
  };
}
