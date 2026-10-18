import { AlertCircle, Library, Pencil, Plus, Trash2, X } from "lucide-react";
import { useEffect, useState } from "react";

import { listen } from "@tauri-apps/api/event";

import { LibraryDetail } from "@/components/library/LibraryDetail";
import { LibraryNameDialog } from "@/components/library/LibraryNameDialog";
import { Button } from "@/components/ui/Button";
import { Dialog } from "@/components/ui/Dialog";
import { EmptyState } from "@/components/ui/EmptyState";
import { IconButton } from "@/components/ui/IconButton";
import { useT } from "@/i18n";
import { isTauriRuntime } from "@/lib/tauri";
import { formatBytes, formatDateTime } from "@/lib/utils";
import { useLibrariesStore } from "@/stores/libraries";
import type { LibrarySummary } from "@/types";

/** 知识库一级页面：创建、列出、重命名、删除本地知识库。 */
export function LibraryView() {
  const items = useLibrariesStore((state) => state.items);
  const loaded = useLibrariesStore((state) => state.loaded);
  const loading = useLibrariesStore((state) => state.loading);
  const error = useLibrariesStore((state) => state.error);
  const load = useLibrariesStore((state) => state.load);
  const create = useLibrariesStore((state) => state.create);
  const rename = useLibrariesStore((state) => state.rename);
  const remove = useLibrariesStore((state) => state.remove);
  const clearError = useLibrariesStore((state) => state.clearError);
  const t = useT();

  const [createOpen, setCreateOpen] = useState(false);
  const [renameTarget, setRenameTarget] = useState<LibrarySummary | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<LibrarySummary | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [openedLibraryId, setOpenedLibraryId] = useState<string | null>(null);

  useEffect(() => {
    void load();
  }, [load]);

  // 文件监听器触发的数据变化：刷新统计与列表
  useEffect(() => {
    if (!isTauriRuntime()) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen("library://changed", () => {
      if (!disposed) void load();
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [load]);

  const openedLibrary = items.find((item) => item.id === openedLibraryId) ?? null;

  if (openedLibrary) {
    return <LibraryDetail library={openedLibrary} onBack={() => setOpenedLibraryId(null)} />;
  }

  const isEmpty = loaded && items.length === 0;

  return (
    <div className="mx-auto w-full max-w-5xl px-8 py-8">
      <header className="flex items-end justify-between gap-4">
        <div>
          <h1 className="text-ink text-xl font-semibold tracking-tight">{t("知识库")}</h1>
          <p className="text-ink-muted mt-1 text-[13px]">
            {t("每个知识库独立索引；所有内容只保存在这台电脑上。")}
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus size={16} aria-hidden="true" />
          {t("新建知识库")}
        </Button>
      </header>

      {error ? (
        <div
          role="alert"
          className="mt-5 flex items-start gap-2 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2.5 text-[13px] text-red-600 dark:text-red-300"
        >
          <AlertCircle size={16} className="mt-0.5 shrink-0" aria-hidden="true" />
          <span className="flex-1">{error}</span>
          <button
            type="button"
            onClick={clearError}
            aria-label={t("关闭错误提示")}
            className="rounded p-0.5 hover:bg-red-500/10"
          >
            <X size={14} />
          </button>
        </div>
      ) : null}

      {loading && !loaded ? <LibrarySkeletons /> : null}

      {isEmpty ? (
        <EmptyState
          className="py-24"
          icon={<Library size={22} aria-hidden="true" />}
          title={t("还没有知识库")}
          description={t(
            "创建一个知识库，然后把 PDF、EPUB、Word、Markdown 或网页拖进来。索引、向量与问答记录全部保存在本机。",
          )}
        >
          <Button onClick={() => setCreateOpen(true)}>
            <Plus size={16} aria-hidden="true" />
            {t("创建第一个知识库")}
          </Button>
        </EmptyState>
      ) : null}

      {items.length > 0 ? (
        <ul
          className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3"
          aria-label={t("知识库列表")}
        >
          {items.map((item) => (
            <li key={item.id}>
              <LibraryCard
                library={item}
                onOpen={() => setOpenedLibraryId(item.id)}
                onRename={() => setRenameTarget(item)}
                onDelete={() => setDeleteTarget(item)}
              />
            </li>
          ))}
        </ul>
      ) : null}

      {createOpen ? (
        <LibraryNameDialog
          open
          title={t("新建知识库")}
          description={t("知识库用于隔离不同主题的文档与索引。")}
          confirmLabel={t("创建")}
          onSubmit={create}
          onClose={() => setCreateOpen(false)}
        />
      ) : null}

      {renameTarget ? (
        <LibraryNameDialog
          open
          title={t("重命名知识库")}
          initialName={renameTarget.name}
          confirmLabel={t("保存")}
          onSubmit={(name) => rename(renameTarget.id, name)}
          onClose={() => setRenameTarget(null)}
        />
      ) : null}

      <Dialog
        open={deleteTarget !== null}
        title={t("删除知识库？")}
        description={t("「{name}」里的文档、索引与问答记录都会从本机删除，且无法恢复。", {
          name: deleteTarget?.name ?? "",
        })}
        onClose={() => setDeleteTarget(null)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setDeleteTarget(null)} disabled={deleting}>
              {t("取消")}
            </Button>
            <Button
              variant="danger"
              disabled={deleting}
              onClick={async () => {
                if (!deleteTarget) return;
                setDeleting(true);
                const success = await remove(deleteTarget.id);
                setDeleting(false);
                if (success) setDeleteTarget(null);
              }}
            >
              {deleting ? t("删除中…") : t("删除")}
            </Button>
          </>
        }
      />
    </div>
  );
}

interface LibraryCardProps {
  /** 知识库数据。 */
  library: LibrarySummary;
  /** 打开知识库详情。 */
  onOpen: () => void;
  /** 点击重命名。 */
  onRename: () => void;
  /** 点击删除。 */
  onDelete: () => void;
}

/** 单个知识库卡片。 */
function LibraryCard({ library, onOpen, onRename, onDelete }: LibraryCardProps) {
  const t = useT();

  return (
    <article
      role="button"
      tabIndex={0}
      onClick={onOpen}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onOpen();
        }
      }}
      className="group border-line bg-surface hover:border-brand/40 focus-visible:outline-brand flex h-full cursor-pointer flex-col rounded-xl border p-4 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2"
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex min-w-0 items-center gap-2">
          <span className="text-brand bg-brand-soft grid size-7 shrink-0 place-items-center rounded-md">
            <Library size={14} aria-hidden="true" />
          </span>
          <h2 className="text-ink truncate text-sm font-semibold" title={library.name}>
            {library.name}
          </h2>
        </div>
        <div className="flex shrink-0 gap-0.5 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100">
          <IconButton
            label={t("重命名 {name}", { name: library.name })}
            onClick={(event) => {
              event.stopPropagation();
              onRename();
            }}
          >
            <Pencil size={14} />
          </IconButton>
          <IconButton
            label={t("删除 {name}", { name: library.name })}
            onClick={(event) => {
              event.stopPropagation();
              onDelete();
            }}
          >
            <Trash2 size={14} />
          </IconButton>
        </div>
      </div>

      <dl className="mt-4 grid grid-cols-3 gap-2 text-center">
        <div className="bg-surface-raised rounded-lg py-2">
          <dt className="text-ink-faint text-[10px]">{t("文档")}</dt>
          <dd className="text-ink mt-0.5 text-sm font-semibold tabular-nums">
            {library.documentCount}
          </dd>
        </div>
        <div className="bg-surface-raised rounded-lg py-2">
          <dt className="text-ink-faint text-[10px]">{t("分块")}</dt>
          <dd className="text-ink mt-0.5 text-sm font-semibold tabular-nums">
            {library.chunkCount}
          </dd>
        </div>
        <div className="bg-surface-raised rounded-lg py-2">
          <dt className="text-ink-faint text-[10px]">{t("占用")}</dt>
          <dd className="text-ink mt-0.5 text-sm font-semibold tabular-nums">
            {formatBytes(library.sizeBytes)}
          </dd>
        </div>
      </dl>

      <p className="text-ink-faint mt-3 text-[11px]">
        {t("更新于 {time}", { time: formatDateTime(library.updatedAt) })}
      </p>
    </article>
  );
}

/** 加载中的骨架屏。 */
function LibrarySkeletons() {
  return (
    <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3" aria-hidden="true">
      {[0, 1, 2].map((index) => (
        <div
          key={index}
          className="border-line bg-surface h-40 animate-pulse rounded-xl border p-4"
        >
          <div className="bg-surface-raised h-4 w-1/2 rounded" />
          <div className="bg-surface-raised mt-5 h-14 rounded-lg" />
          <div className="bg-surface-raised mt-3 h-3 w-2/5 rounded" />
        </div>
      ))}
    </div>
  );
}
