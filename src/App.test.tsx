import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "@/App";

const listLibraries = vi.fn();
const createLibrary = vi.fn();
const renameLibrary = vi.fn();
const deleteLibrary = vi.fn();
const listDocuments = vi.fn();
const listIngestJobs = vi.fn();
const getIngestQueueState = vi.fn();

vi.mock("@/lib/tauri", () => ({
  isTauriRuntime: () => true,
  getAppInfo: vi.fn().mockResolvedValue({
    name: "Lumos",
    version: "0.1.0",
    dataDir: "C:/Users/test/AppData/Roaming/com.daiyi.lumos",
    dataDirDisplay: "~/AppData/Roaming/com.daiyi.lumos",
    platform: "windows",
  }),
  listLibraries: (...args: unknown[]) => listLibraries(...args),
  createLibrary: (...args: unknown[]) => createLibrary(...args),
  renameLibrary: (...args: unknown[]) => renameLibrary(...args),
  deleteLibrary: (...args: unknown[]) => deleteLibrary(...args),
  listDocuments: (...args: unknown[]) => listDocuments(...args),
  listIngestJobs: (...args: unknown[]) => listIngestJobs(...args),
  getIngestQueueState: (...args: unknown[]) => getIngestQueueState(...args),
  importPaths: vi.fn().mockResolvedValue(0),
  pauseIngestQueue: vi.fn().mockResolvedValue(true),
  resumeIngestQueue: vi.fn().mockResolvedValue(false),
  cancelIngestJob: vi.fn().mockResolvedValue(undefined),
  deleteDocument: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => undefined),
}));

beforeEach(() => {
  listLibraries.mockReset().mockResolvedValue([]);
  createLibrary.mockReset();
  renameLibrary.mockReset();
  deleteLibrary.mockReset();
  listDocuments.mockReset().mockResolvedValue([]);
  listIngestJobs.mockReset().mockResolvedValue([]);
  getIngestQueueState.mockReset().mockResolvedValue(false);
});

describe("AppShell", () => {
  it("渲染品牌、导航与空状态", async () => {
    render(<App />);

    expect(screen.getByRole("img", { name: "Lumos" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "知识库" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "问答" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "设置" })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "还没有知识库" })).toBeInTheDocument();

    // 等待异步的应用信息加载完成，避免 act() 警告
    expect(await screen.findByText(/Lumos v0\.1\.0/)).toBeInTheDocument();
  });

  it("点击导航可以在三个一级页面之间切换", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole("button", { name: "问答" }));
    expect(screen.getByRole("heading", { name: "从一个问题开始" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "设置" }));
    expect(screen.getByRole("heading", { name: "设置" })).toBeInTheDocument();

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
    createLibrary.mockResolvedValue({
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
    expect(createLibrary).toHaveBeenCalledWith("合同条款");
  });

  it("重命名知识库会更新列表", async () => {
    const user = userEvent.setup();
    listLibraries.mockResolvedValue([
      {
        id: "lib-1",
        name: "旧名字",
        createdAt: "2026-10-18T00:00:00.000Z",
        updatedAt: "2026-10-18T00:00:00.000Z",
        embeddingModel: "BAAI/bge-small-zh-v1.5",
        chunkConfig: "{}",
        documentCount: 2,
        chunkCount: 8,
        sizeBytes: 4096,
      },
    ]);
    renameLibrary.mockResolvedValue({
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

  it("删除知识库需要确认，确认后从列表移除", async () => {
    const user = userEvent.setup();
    listLibraries.mockResolvedValue([
      {
        id: "lib-1",
        name: "待删除",
        createdAt: "2026-10-18T00:00:00.000Z",
        updatedAt: "2026-10-18T00:00:00.000Z",
        embeddingModel: "BAAI/bge-small-zh-v1.5",
        chunkConfig: "{}",
        documentCount: 0,
        chunkCount: 0,
        sizeBytes: 0,
      },
    ]);
    deleteLibrary.mockResolvedValue(undefined);

    render(<App />);
    await screen.findByRole("heading", { name: "待删除" });

    await user.click(screen.getByRole("button", { name: "删除 待删除" }));
    expect(await screen.findByRole("dialog", { name: "删除知识库？" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "删除" }));

    expect(await screen.findByRole("heading", { name: "还没有知识库" })).toBeInTheDocument();
    expect(deleteLibrary).toHaveBeenCalledWith("lib-1");
  });

  it("打开知识库后展示文档列表", async () => {
    const user = userEvent.setup();
    listLibraries.mockResolvedValue([
      {
        id: "lib-1",
        name: "论文库",
        createdAt: "2026-10-18T00:00:00.000Z",
        updatedAt: "2026-10-18T00:00:00.000Z",
        embeddingModel: "BAAI/bge-small-zh-v1.5",
        chunkConfig: "{}",
        documentCount: 1,
        chunkCount: 12,
        sizeBytes: 2048,
      },
    ]);
    listDocuments.mockResolvedValue([
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
    expect(listDocuments).toHaveBeenCalledWith("lib-1");
  });
});
