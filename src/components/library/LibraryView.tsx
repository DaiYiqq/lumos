import { AlertCircle, Library, Pencil, Plus, Trash2, X } from "lucide-react";
import { useEffect, useState } from "react";

import { LibraryNameDialog } from "@/components/library/LibraryNameDialog";
import { Button } from "@/components/ui/Button";
import { Dialog } from "@/components/ui/Dialog";
import { EmptyState } from "@/components/ui/EmptyState";
import { IconButton } from "@/components/ui/IconButton";
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

  const [createOpen, setCreateOpen] = useState(false);
  const [renameTarget, setRenameTarget] = useState<LibrarySummary | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<LibrarySummary | null>(null);
  const [deleting, setDeleting] = useState(false);

  useEffect(() => {
    void load();
  }, [load]);

  const isEmpty = loaded && items.length === 0;

  return (
    <div className="mx-auto w-full max-w-5xl px-8 py-8">
      <header className="flex items-end justify-between gap-4">
        <div>
          <h1 className="text-ink text-xl font-semibold tracking-tight">知识库</h1>
          <p className="text-ink-muted mt-1 text-[13px]">
            每个知识库独立索引；所有内容只保存在这台电脑上。
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus size={16} aria-hidden="true" />
          新建知识库
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
            aria-label="关闭错误提示"
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
          title="还没有知识库"
          description="创建一个知识库，然后把 PDF、EPUB、Word、Markdown 或网页拖进来。索引、向量与问答记录全部保存在本机。"
        >
          <Button onClick={() => setCreateOpen(true)}>
            <Plus size={16} aria-hidden="true" />
            创建第一个知识库
          </Button>
        </EmptyState>
      ) : null}

      {items.length > 0 ? (
        <ul
          className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3"
          aria-label="知识库列表"
        >
          {items.map((item) => (
            <li key={item.id}>
              <LibraryCard
                library={item}
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
          title="新建知识库"
          description="知识库用于隔离不同主题的文档与索引。"
          confirmLabel="创建"
          onSubmit={create}
          onClose={() => setCreateOpen(false)}
        />
      ) : null}

      {renameTarget ? (
        <LibraryNameDialog
          open
          title="重命名知识库"
          initialName={renameTarget.name}
          confirmLabel="保存"
          onSubmit={(name) => rename(renameTarget.id, name)}
          onClose={() => setRenameTarget(null)}
        />
      ) : null}

      <Dialog
        open={deleteTarget !== null}
        title="删除知识库？"
        description={`「${deleteTarget?.name ?? ""}」里的文档、索引与问答记录都会从本机删除，且无法恢复。`}
        onClose={() => setDeleteTarget(null)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setDeleteTarget(null)} disabled={deleting}>
              取消
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
              {deleting ? "删除中…" : "删除"}
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
  /** 点击重命名。 */
  onRename: () => void;
  /** 点击删除。 */
  onDelete: () => void;
}

/** 单个知识库卡片。 */
function LibraryCard({ library, onRename, onDelete }: LibraryCardProps) {
  return (
    <article className="group border-line bg-surface hover:border-brand/40 flex h-full flex-col rounded-xl border p-4 transition-colors">
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
          <IconButton label={`重命名 ${library.name}`} onClick={onRename}>
            <Pencil size={14} />
          </IconButton>
          <IconButton label={`删除 ${library.name}`} onClick={onDelete}>
            <Trash2 size={14} />
          </IconButton>
        </div>
      </div>

      <dl className="mt-4 grid grid-cols-3 gap-2 text-center">
        <div className="bg-surface-raised rounded-lg py-2">
          <dt className="text-ink-faint text-[10px]">文档</dt>
          <dd className="text-ink mt-0.5 text-sm font-semibold tabular-nums">
            {library.documentCount}
          </dd>
        </div>
        <div className="bg-surface-raised rounded-lg py-2">
          <dt className="text-ink-faint text-[10px]">分块</dt>
          <dd className="text-ink mt-0.5 text-sm font-semibold tabular-nums">
            {library.chunkCount}
          </dd>
        </div>
        <div className="bg-surface-raised rounded-lg py-2">
          <dt className="text-ink-faint text-[10px]">占用</dt>
          <dd className="text-ink mt-0.5 text-sm font-semibold tabular-nums">
            {formatBytes(library.sizeBytes)}
          </dd>
        </div>
      </dl>

      <p className="text-ink-faint mt-3 text-[11px]">更新于 {formatDateTime(library.updatedAt)}</p>
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
