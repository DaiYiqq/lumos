import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import App from "@/App";

vi.mock("@/lib/tauri", () => ({
  isTauriRuntime: () => true,
  getAppInfo: vi.fn().mockResolvedValue({
    name: "Lumos",
    version: "0.1.0",
    dataDir: "C:/Users/test/AppData/Roaming/com.daiyi.lumos",
    dataDirDisplay: "~/AppData/Roaming/com.daiyi.lumos",
    platform: "windows",
  }),
}));

describe("AppShell", () => {
  it("渲染品牌、导航与空状态", async () => {
    render(<App />);

    expect(screen.getByRole("img", { name: "Lumos" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "知识库" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "问答" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "设置" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "还没有知识库" })).toBeInTheDocument();

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
    expect(screen.getByRole("heading", { name: "还没有知识库" })).toBeInTheDocument();
  });

  it("状态栏显示应用版本与运行平台", async () => {
    render(<App />);

    expect(await screen.findByText(/Lumos v0\.1\.0 · windows/)).toBeInTheDocument();
    expect(await screen.findByText(/com\.daiyi\.lumos/)).toBeInTheDocument();
  });
});
