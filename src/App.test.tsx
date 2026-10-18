import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "@/App";
import { useChatStore } from "@/stores/chat";
import { useDocumentsStore } from "@/stores/documents";
import { useLibrariesStore } from "@/stores/libraries";
import { useUiStore } from "@/stores/ui";

const api = {
  listLibraries: vi.fn(),
  createLibrary: vi.fn(),
  renameLibrary: vi.fn(),
  deleteLibrary: vi.fn(),
  listDocuments: vi.fn(),
  listIngestJobs: vi.fn(),
  getIngestQueueState: vi.fn(),
  listConversations: vi.fn(),
  getConversation: vi.fn(),
  streamChat: vi.fn(),
  apiKeyStatus: vi.fn(),
  providersDetect: vi.fn(),
  embeddingStatus: vi.fn(),
  settingsGet: vi.fn(),
};

vi.mock("@/lib/tauri", () => ({
  isTauriRuntime: () => true,
  getAppInfo: vi.fn().mockResolvedValue({
    name: "Lumos",
    version: "0.1.0",
    dataDir: "C:/Users/test/AppData/Roaming/com.daiyi.lumos",
    dataDirDisplay: "~/AppData/Roaming/com.daiyi.lumos",
    platform: "windows",
  }),
  listLibraries: (...args: unknown[]) => api.listLibraries(...args),
  createLibrary: (...args: unknown[]) => api.createLibrary(...args),
  renameLibrary: (...args: unknown[]) => api.renameLibrary(...args),
  deleteLibrary: (...args: unknown[]) => api.deleteLibrary(...args),
  listDocuments: (...args: unknown[]) => api.listDocuments(...args),
  listIngestJobs: (...args: unknown[]) => api.listIngestJobs(...args),
  getIngestQueueState: (...args: unknown[]) => api.getIngestQueueState(...args),
  importPaths: vi.fn().mockResolvedValue(0),
  pauseIngestQueue: vi.fn().mockResolvedValue(true),
  resumeIngestQueue: vi.fn().mockResolvedValue(false),
  cancelIngestJob: vi.fn().mockResolvedValue(undefined),
  deleteDocument: vi.fn().mockResolvedValue(undefined),
  listConversations: (...args: unknown[]) => api.listConversations(...args),
  createConversation: vi.fn(),
  getConversation: (...args: unknown[]) => api.getConversation(...args),
  renameConversation: vi.fn().mockResolvedValue(undefined),
  deleteConversation: vi.fn().mockResolvedValue(undefined),
  openDataDir: vi.fn().mockResolvedValue(undefined),
  streamChat: (...args: unknown[]) => api.streamChat(...args),
  completeChat: vi.fn(),
  apiKeyStatus: (...args: unknown[]) => api.apiKeyStatus(...args),
  apiKeySet: vi.fn().mockResolvedValue({ configured: true, source: "keyring" }),
  apiKeyClear: vi.fn().mockResolvedValue({ configured: false, source: "none" }),
  providersDetect: (...args: unknown[]) => api.providersDetect(...args),
  settingsGet: (...args: unknown[]) => api.settingsGet(...args),
  settingsSet: vi.fn().mockResolvedValue(undefined),
  embeddingStatus: (...args: unknown[]) => api.embeddingStatus(...args),
  embeddingBackfill: vi.fn().mockResolvedValue(0),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => undefined),
}));

const library = {
  id: "lib-1",
  name: "论文库",
  createdAt: "2026-10-18T00:00:00.000Z",
  updatedAt: "2026-10-18T00:00:00.000Z",
  embeddingModel: "BAAI/bge-small-zh-v1.5",
  chunkConfig: "{}",
  documentCount: 1,
  chunkCount: 12,
  sizeBytes: 2048,
};

const citation = {
  index: 1,
  chunkId: 9,
  documentId: "doc-1",
  documentTitle: "合同违约金条款",
  documentPath: "/tmp/contract.docx",
  pageNo: 3,
  sectionPath: "第九条",
  content: "违约金上限为合同总额的百分之二十。",
  score: 0.03,
};

beforeEach(() => {
  // zustand store 在测试间是共享的，逐个重置保证用例互不影响
  useLibrariesStore.setState({ items: [], loaded: false, loading: false, error: null });
  useDocumentsStore.setState({
    documents: [],
    jobs: [],
    loading: false,
    paused: false,
    error: null,
  });
  useUiStore.setState({ activeView: "library", selectedLibraryId: null });
  useChatStore.setState({
    libraryId: null,
    conversations: [],
    activeId: null,
    messages: [],
    citations: [],
    selectedCitation: null,
    streaming: false,
    provider: null,
    providerLabel: null,
    notice: null,
    error: null,
    loading: false,
  });

  api.listLibraries.mockReset().mockResolvedValue([]);
  api.createLibrary.mockReset();
  api.renameLibrary.mockReset();
  api.deleteLibrary.mockReset();
  api.listDocuments.mockReset().mockResolvedValue([]);
  api.listIngestJobs.mockReset().mockResolvedValue([]);
  api.getIngestQueueState.mockReset().mockResolvedValue(false);
  api.listConversations.mockReset().mockResolvedValue([]);
  api.getConversation.mockReset();
  api.streamChat.mockReset();
  api.apiKeyStatus.mockReset().mockResolvedValue({ configured: false, source: "none" });
  api.providersDetect.mockReset().mockResolvedValue({
    statuses: [
      { id: "deepseek", available: false, selected: false, detail: "未配置 API Key" },
      { id: "ollama", available: false, selected: false, detail: "未检测到 Ollama" },
      { id: "none", available: true, selected: true, detail: "离线关键词模式，永远可用" },
    ],
    selected: "none",
  });
  api.embeddingStatus
    .mockReset()
    .mockResolvedValue({
      state: "ready",
      model: "BAAI/bge-small-zh-v1.5",
      dimension: 512,
      message: null,
    });
  api.settingsGet.mockReset().mockResolvedValue(null);
});

describe("应用外壳", () => {
  it("渲染品牌、导航与知识库空状态", async () => {
    render(<App />);

    expect(screen.getByRole("img", { name: "Lumos" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "知识库" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "问答" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "设置" })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "还没有知识库" })).toBeInTheDocument();
    expect(await screen.findByText(/Lumos v0\.1\.0/)).toBeInTheDocument();
  });

  it("导航可以在三个一级页面之间切换", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole("button", { name: "问答" }));
    expect(await screen.findByRole("heading", { name: "先创建一个知识库" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "设置" }));
    expect(await screen.findByRole("heading", { name: "设置" })).toBeInTheDocument();
    expect((await screen.findAllByText(/离线关键词模式/)).length).toBeGreaterThan(0);

    await user.click(screen.getByRole("button", { name: "知识库" }));
    expect(await screen.findByRole("heading", { name: "还没有知识库" })).toBeInTheDocument();
  });

  it("状态栏显示应用版本与运行平台", async () => {
    render(<App />);

    expect(await screen.findByText(/Lumos v0\.1\.0 · windows/)).toBeInTheDocument();
    expect(await screen.findByText(/com\.daiyi\.lumos/)).toBeInTheDocument();
  });
});

describe("知识库", () => {
  it("创建知识库后出现在列表中", async () => {
    const user = userEvent.setup();
    api.createLibrary.mockResolvedValue({
      id: "lib-1",
      name: "合同条款",
      createdAt: "2026-10-18T00:00:00.000Z",
      updatedAt: "2026-10-18T00:00:00.000Z",
      embeddingModel: "BAAI/bge-small-zh-v1.5",
      chunkConfig: "{}",
    });

    render(<App />);
    await screen.findByRole("heading", { name: "还没有知识库" });

    await user.click(screen.getByRole("button", { name: "新建知识库" }));
    await user.type(await screen.findByLabelText("知识库名称"), "合同条款");
    await user.click(screen.getByRole("button", { name: "创建" }));

    expect(await screen.findByRole("heading", { name: "合同条款" })).toBeInTheDocument();
    expect(api.createLibrary).toHaveBeenCalledWith("合同条款");
  });

  it("重命名知识库会更新列表", async () => {
    const user = userEvent.setup();
    api.listLibraries.mockResolvedValue([{ ...library, name: "旧名字" }]);
    api.renameLibrary.mockResolvedValue({
      id: "lib-1",
      name: "新名字",
      createdAt: "2026-10-18T00:00:00.000Z",
      updatedAt: "2026-10-18T01:00:00.000Z",
      embeddingModel: "BAAI/bge-small-zh-v1.5",
      chunkConfig: "{}",
    });

    render(<App />);
    await screen.findByRole("heading", { name: "旧名字" });

    await user.click(screen.getByRole("button", { name: "重命名 旧名字" }));
    const input = await screen.findByLabelText("知识库名称");
    await user.clear(input);
    await user.type(input, "新名字");
    await user.click(screen.getByRole("button", { name: "保存" }));

    expect(await screen.findByRole("heading", { name: "新名字" })).toBeInTheDocument();
  });

  it("打开知识库后展示文档列表", async () => {
    const user = userEvent.setup();
    api.listLibraries.mockResolvedValue([library]);
    api.listDocuments.mockResolvedValue([
      {
        id: "doc-1",
        libraryId: "lib-1",
        path: "/papers/contrastive.pdf",
        title: "对比学习综述",
        mime: "application/pdf",
        size: 2048,
        pageCount: 12,
        status: "ready",
        errorMsg: null,
        createdAt: "2026-10-18T00:00:00.000Z",
        updatedAt: "2026-10-18T00:00:00.000Z",
        chunkCount: 12,
      },
    ]);

    render(<App />);
    await screen.findByRole("heading", { name: "论文库" });
    await user.click(screen.getByRole("button", { name: /^论文库/ }));

    expect(await screen.findByText("对比学习综述")).toBeInTheDocument();
    expect(screen.getByText("已就绪")).toBeInTheDocument();
  });
});

describe("问答", () => {
  it("发送问题后流式显示回答，点击引用可查看原文", async () => {
    const user = userEvent.setup();
    api.listLibraries.mockResolvedValue([library]);
    api.listConversations.mockResolvedValue([]);
    api.streamChat.mockImplementation(
      async (
        _libraryId: string,
        _question: string,
        _conversationId: string | null,
        _history: unknown,
        onEvent: (event: unknown) => void,
      ) => {
        onEvent({ type: "conversation", id: "conv-1", title: "违约金问题" });
        onEvent({ type: "provider", provider: "deepseek", label: "DeepSeek" });
        onEvent({ type: "citations", citations: [citation] });
        onEvent({ type: "delta", text: "违约金上限为合同总额的百分之二十 " });
        onEvent({ type: "delta", text: "[1]。" });
        onEvent({ type: "done", provider: "deepseek" });
      },
    );

    render(<App />);
    await user.click(await screen.findByRole("button", { name: "问答" }));

    const starter = await screen.findByRole("button", {
      name: /合同里对违约金上限/,
    });
    await user.click(starter);

    expect((await screen.findAllByText(/百分之二十/)).length).toBeGreaterThan(0);
    expect(await screen.findByText("DeepSeek")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "查看引用 1" }));
    expect(await screen.findByText("合同违约金条款")).toBeInTheDocument();
    expect(screen.getByText(/第九条/)).toBeInTheDocument();
  });

  it("离线模式会显示明确的模式徽标", async () => {
    const user = userEvent.setup();
    api.listLibraries.mockResolvedValue([library]);
    api.streamChat.mockImplementation(
      async (
        _libraryId: string,
        _question: string,
        _conversationId: string | null,
        _history: unknown,
        onEvent: (event: unknown) => void,
      ) => {
        onEvent({ type: "conversation", id: "conv-1", title: "离线" });
        onEvent({ type: "provider", provider: "none", label: "离线关键词模式" });
        onEvent({ type: "delta", text: "离线关键词模式：未配置 AI 模型。" });
        onEvent({ type: "done", provider: "none" });
      },
    );

    render(<App />);
    await user.click(await screen.findByRole("button", { name: "问答" }));
    await user.click(await screen.findByRole("button", { name: /对比学习做医学图像分割/ }));

    expect((await screen.findAllByText("离线关键词模式")).length).toBeGreaterThan(0);
    expect(await screen.findByText(/未配置 AI 模型/)).toBeInTheDocument();
  });
});
