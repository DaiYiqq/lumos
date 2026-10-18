import { Download, Sparkles } from "lucide-react";

/**
 * 仅在线 Demo 构建显示的提示条。
 * 说明数据只存在于浏览器内，并引导用户下载桌面版。
 */
export function DemoBanner() {
  return (
    <div className="bg-brand-soft text-ink border-brand/20 flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1 border-b px-4 py-2 text-[12px]">
      <span className="text-brand inline-flex items-center gap-1.5 font-semibold">
        <Sparkles size={13} aria-hidden="true" />
        Online Demo
      </span>
      <span className="text-ink-muted">
        浏览器内模拟数据，不会上传文件；刷新即重置。
        <span className="text-ink-faint"> Browser-only mock data, nothing is uploaded.</span>
      </span>
      <a
        href="https://github.com/DaiYiqq/lumos/releases/latest"
        target="_blank"
        rel="noreferrer"
        className="text-brand ml-auto inline-flex items-center gap-1 font-medium underline underline-offset-2"
      >
        <Download size={12} aria-hidden="true" />
        下载桌面版 / Download
      </a>
    </div>
  );
}
