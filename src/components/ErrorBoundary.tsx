import { Component } from "react";
import type { ErrorInfo, ReactNode } from "react";

import { FolderOpen, RotateCcw } from "lucide-react";

import { Button } from "@/components/ui/Button";
import { openDataDir } from "@/lib/tauri";

interface ErrorBoundaryState {
  /** 是否捕获到渲染异常。 */
  hasError: boolean;
  /** 异常信息（仅用于展示摘要）。 */
  message: string;
}

/** 全局错误边界：渲染异常时展示可恢复页面，而不是白屏。 */
export class ErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false, message: "" };
  }

  static getDerivedStateFromError(error: unknown): ErrorBoundaryState {
    return {
      hasError: true,
      message: error instanceof Error ? error.message : String(error),
    };
  }

  override componentDidCatch(error: unknown, info: ErrorInfo): void {
    console.error("界面渲染异常", error, info.componentStack);
  }

  override render(): ReactNode {
    if (!this.state.hasError) return this.props.children;

    return (
      <div className="bg-canvas text-ink flex h-full items-center justify-center p-8">
        <div className="border-line bg-surface w-full max-w-lg rounded-xl border p-6 text-center">
          <h1 className="text-lg font-semibold">界面遇到了一点问题</h1>
          <p className="text-ink-muted mt-2 text-[13px] leading-relaxed">
            数据没有丢失。可以先尝试重新加载界面；如果反复出现，请在数据目录的 logs
            中查看日志并反馈。
          </p>
          <p className="bg-surface-raised text-ink-faint mt-3 rounded-lg px-3 py-2 text-left font-mono text-[11px] break-all">
            {this.state.message || "未知错误"}
          </p>
          <div className="mt-5 flex justify-center gap-2">
            <Button onClick={() => window.location.reload()}>
              <RotateCcw size={14} aria-hidden="true" />
              重新加载
            </Button>
            <Button
              variant="secondary"
              onClick={() => {
                void openDataDir().catch(() => undefined);
              }}
            >
              <FolderOpen size={14} aria-hidden="true" />
              打开数据目录
            </Button>
          </div>
        </div>
      </div>
    );
  }
}
