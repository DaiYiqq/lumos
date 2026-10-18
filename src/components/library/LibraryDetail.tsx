import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  AlertCircle,
  ArrowLeft,
  FileText,
  FolderPlus,
  Loader2,
  Pause,
  Play,
  RefreshCw,
  Trash2,
  Upload,
  X,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";

import { Button } from "@/components/ui/Button";
import { EmptyState } from "@/components/ui/EmptyState";
import { IconButton } from "@/components/ui/IconButton";
import { DOCUMENT_FILTER } from "@/lib/fileTypes";
import { isTauriRuntime, importPaths as callImportPaths } from "@/lib/tauri";
import { formatBytes, formatDateTime } from "@/lib/utils";
import { useDocumentsStore } from "@/stores/documents";
import { useLibrariesStore } from "@/stores/libraries";
import type { IngestProgress, IngestJob, LibrarySummary } from "@/types";

interface LibraryDetailProps {
  /** 当前知识库。 */
  library: LibrarySummary;
  /** 返回知识库列表。 */
  onBack: () => void;
}

/** 知识库详情：导入文档、查看索引进度、管理文档。 */
export function LibraryDetail({ library, onBack }: LibraryDetailProps) {
  const documents = useDocumentsStore((state) => state.documents);
  const jobs = useDocumentsStore((state) => state.jobs);
  const loading = useDocumentsStore((state) => state.loading);
  const paused = useDocumentsStore((state) => state.paused);
  const error = useDocumentsStore((state) => state.error);
  const load = useDocumentsStore((state) => state.load);
  const clear = useDocumentsStore((state) => state.clear);
  const setPaused = useDocumentsStore((state) => state.setPaused);
  const cancelJob = useDocumentsStore((state) => state.cancelJob);
  const removeDocument = useDocumentsStore((state) => state.removeDocument);
  const loadLibraries = useLibrariesStore((state) => state.load);

  const [importing, setImporting] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    await load(library.id);
    await loadLibraries();
  }, [library.id, load, loadLibraries]);

  useEffect(() => {
    void refresh();
    return () => clear();
  }, [refresh, clear]);

  // 订阅后端进度事件：每完成一个文件刷新一次列表
  useEffect(() => {
    if (!isTauriRuntime()) return;

    let disposed = false;
    let timer: ReturnType<typeof setTimeout> | null = null;
    let unlisten: (() => void) | undefined;

    void listen<IngestProgress>("ingest://progress", (event) => {
      if (event.payload.libraryId !== library.id) return;
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        if (!disposed) void refresh();
      }, 250);
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });

    return () => {
      disposed = true;
      if (timer) clearTimeout(timer);
      unlisten?.();
    };
  }, [library.id, refresh]);

  const runningJob = useMemo(() => jobs.find((job) => job.status === "running"), [jobs]);
  const pendingCount = useMemo(() => jobs.filter((job) => job.status === "pending").length, [jobs]);
  const lastFailure = useMemo(() => jobs.find((job) => job.status === "failed") ?? null, [jobs]);

  const handleImport = async (directory: boolean): Promise<void> => {
    if (!isTauriRuntime()) {
      setNotice("浏览器预览模式：请在桌面应用里导入文档");
      return;
    }

    setImporting(true);
    setNotice(null);
    try {
      const selection = await open(
        directory
          ? { directory: true, multiple: false, title: "选择要导入的文件夹" }
          : {
              multiple: true,
              directory: false,
              title: "选择要导入的文档",
              filters: [DOCUMENT_FILTER],
            },
      );
      if (!selection) return;

      const paths = Array.isArray(selection) ? selection : [selection];
      if (paths.length === 0) return;

      const created = await importPathsSafely(library.id, paths);
      setNotice(created > 0 ? `已加入 ${created} 个导入任务` : "没有新的可导入文件");
      await refresh();
    } catch (importError) {
      setNotice(describeError(importError));
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="mx-auto w-full max-w-5xl px-8 py-8">
      <header className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex min-w-0 items-center gap-3">
          <IconButton label="返回知识库列表" onClick={onBack} className="border-line border">
            <ArrowLeft size={16} />
          </IconButton>
          <div className="min-w-0">
            <h1 className="text-ink truncate text-xl font-semibold tracking-tight">
              {library.name}
            </h1>
            <p className="text-ink-muted mt-0.5 text-[12px]">
              {library.documentCount} 篇文档 · {library.chunkCount} 个分块 · 更新于{" "}
              {formatDateTime(library.updatedAt)}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <IconButton
            label="刷新"
            onClick={() => void refresh()}
            className="border-line h-9 w-9 border"
          >
            <RefreshCw size={15} className={loading ? "animate-spin" : undefined} />
          </IconButton>
          {paused ? (
            <Button variant="secondary" onClick={() => void setPaused(false)}>
              <Play size={15} aria-hidden="true" />
              继续导入
            </Button>
          ) : (
            <Button
              variant="secondary"
              onClick={() => void setPaused(true)}
              disabled={!runningJob && pendingCount === 0}
            >
              <Pause size={15} aria-hidden="true" />
              暂停
            </Button>
          )}
          <Button onClick={() => void handleImport(false)} disabled={importing}>
            <Upload size={15} aria-hidden="true" />
            导入文件
          </Button>
          <Button variant="secondary" onClick={() => void handleImport(true)} disabled={importing}>
            <FolderPlus size={15} aria-hidden="true" />
            导入文件夹
          </Button>
        </div>
      </header>

      {notice ? (
        <div className="border-line bg-surface-raised text-ink-muted mt-4 flex items-center gap-2 rounded-lg border px-3 py-2 text-[12px]">
          <span className="flex-1">{notice}</span>
          <IconButton label="关闭提示" onClick={() => setNotice(null)}>
            <X size={13} />
          </IconButton>
        </div>
      ) : null}

      {error ? (
        <div
          role="alert"
          className="mt-4 flex items-start gap-2 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2.5 text-[13px] text-red-600 dark:text-red-300"
        >
          <AlertCircle size={16} className="mt-0.5 shrink-0" aria-hidden="true" />
          <span className="flex-1">{error}</span>
        </div>
      ) : null}

      <QueueBanner
        job={runningJob ?? null}
        pendingCount={pendingCount}
        paused={paused}
        onCancel={cancelJob}
      />

      {lastFailure ? (
        <div className="mt-3 rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-[12px] text-amber-700 dark:text-amber-300">
          最近一次导入失败：{lastFailure.message ?? "未知原因"}（{basename(lastFailure.path)}）
        </div>
      ) : null}

      <section className="mt-6" aria-label="文档列表">
        {documents.length === 0 && !loading ? (
          <EmptyState
            className="py-20"
            icon={<FileText size={22} aria-hidden="true" />}
            title="还没有文档"
            description="导入 PDF、EPUB、Word、Markdown、网页或代码文件。解析与分块都在本机完成，源文件不会被移动或上传。"
          >
            <Button onClick={() => void handleImport(false)}>
              <Upload size={16} aria-hidden="true" />
              导入文件
            </Button>
          </EmptyState>
        ) : (
          <ul className="border-line bg-surface divide-line divide-y overflow-hidden rounded-xl border">
            {documents.map((document) => (
              <li key={document.id} className="flex items-center gap-3 px-4 py-3">
                <FileText size={16} className="text-ink-faint shrink-0" aria-hidden="true" />
                <div className="min-w-0 flex-1">
                  <div className="text-ink truncate text-sm font-medium">{document.title}</div>
                  <div className="text-ink-faint mt-0.5 truncate text-[11px]" title={document.path}>
                    {document.chunkCount} 个分块 · {formatBytes(document.size)}
                    {document.pageCount ? ` · ${document.pageCount} 页` : ""} ·{" "}
                    {formatDateTime(document.updatedAt)}
                  </div>
                </div>
                <StatusBadge status={document.status} />
                <IconButton
                  label={`删除文档 ${document.title}`}
                  onClick={() => void removeDocument(document.id)}
                >
                  <Trash2 size={14} />
                </IconButton>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

interface QueueBannerProps {
  /** 正在运行的任务。 */
  job: IngestJob | null;
  /** 排队中的任务数。 */
  pendingCount: number;
  /** 队列是否暂停。 */
  paused: boolean;
  /** 取消任务。 */
  onCancel: (jobId: string) => Promise<void>;
}

/** 顶部进度条：当前文件 + 队列长度。 */
function QueueBanner({ job, pendingCount, paused, onCancel }: QueueBannerProps) {
  if (!job && pendingCount === 0) return null;

  return (
    <div className="border-line bg-surface mt-4 rounded-xl border px-4 py-3">
      <div className="flex items-center gap-3">
        <Loader2 size={15} className="text-brand animate-spin" aria-hidden="true" />
        <div className="min-w-0 flex-1">
          <p className="text-ink truncate text-[13px] font-medium">
            {job ? `正在解析 ${basename(job.path)}` : paused ? "导入已暂停" : "等待导入"}
          </p>
          <p className="text-ink-faint mt-0.5 text-[11px]">
            {paused ? "恢复后会继续处理队列中的文件" : `队列中还有 ${pendingCount} 个文件`}
          </p>
        </div>
        {job ? (
          <Button variant="ghost" size="sm" onClick={() => void onCancel(job.id)}>
            取消
          </Button>
        ) : null}
      </div>
      <div className="bg-surface-raised mt-2.5 h-1 overflow-hidden rounded-full">
        <div
          className={`bg-brand h-full w-1/3 rounded-full ${paused ? "opacity-40" : "animate-pulse"}`}
        />
      </div>
    </div>
  );
}

interface StatusBadgeProps {
  /** 文档状态。 */
  status: string;
}

/** 文档状态徽标。 */
function StatusBadge({ status }: StatusBadgeProps) {
  const map: Record<string, { label: string; className: string }> = {
    pending: { label: "排队中", className: "bg-surface-raised text-ink-muted" },
    indexing: { label: "解析中", className: "bg-brand-soft text-brand" },
    ready: {
      label: "已就绪",
      className: "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400",
    },
    failed: { label: "失败", className: "bg-red-500/10 text-red-600 dark:text-red-400" },
  };
  const meta = map[status] ?? { label: status, className: "bg-surface-raised text-ink-muted" };

  return (
    <span className={`shrink-0 rounded-full px-2 py-0.5 text-[11px] font-medium ${meta.className}`}>
      {meta.label}
    </span>
  );
}

/** 调用后端导入命令；这里单独包一层便于测试时 mock。 */
async function importPathsSafely(libraryId: string, paths: string[]): Promise<number> {
  return callImportPaths(libraryId, paths);
}

/** 取路径最后一段作为展示名。 */
function basename(path: string): string {
  const normalized = path.replaceAll("\\", "/");
  return normalized.slice(normalized.lastIndexOf("/") + 1) || path;
}

/** 把未知错误转成一句可展示的话。 */
function describeError(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return "操作失败，请重试";
}
