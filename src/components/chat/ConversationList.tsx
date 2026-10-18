import { MessageSquarePlus, Pencil, Trash2 } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/Button";
import { Dialog } from "@/components/ui/Dialog";
import { IconButton } from "@/components/ui/IconButton";
import { Input } from "@/components/ui/Input";
import { cn } from "@/lib/utils";
import type { ConversationSummary } from "@/types";

interface ConversationListProps {
  /** 会话列表。 */
  conversations: ConversationSummary[];
  /** 当前会话。 */
  activeId: string | null;
  /** 是否加载中。 */
  loading: boolean;
  /** 选择会话。 */
  onSelect: (id: string) => void;
  /** 新建对话。 */
  onNew: () => void;
  /** 重命名。 */
  onRename: (id: string, title: string) => Promise<void>;
  /** 删除。 */
  onDelete: (id: string) => Promise<void>;
}

/** 问答页的会话列表。 */
export function ConversationList({
  conversations,
  activeId,
  loading,
  onSelect,
  onNew,
  onRename,
  onDelete,
}: ConversationListProps) {
  const [renameTarget, setRenameTarget] = useState<ConversationSummary | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ConversationSummary | null>(null);
  const [title, setTitle] = useState("");
  const [busy, setBusy] = useState(false);

  return (
    <aside className="border-line bg-surface flex w-60 shrink-0 flex-col border-r">
      <div className="flex items-center justify-between px-3 pt-3 pb-2">
        <span className="text-ink text-[13px] font-semibold">对话</span>
        <IconButton label="新建对话" onClick={onNew}>
          <MessageSquarePlus size={15} />
        </IconButton>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {loading && conversations.length === 0 ? (
          <div className="space-y-1.5 px-1 pt-1" aria-hidden="true">
            {[0, 1, 2].map((index) => (
              <div key={index} className="bg-surface-raised h-9 animate-pulse rounded-lg" />
            ))}
          </div>
        ) : conversations.length === 0 ? (
          <p className="text-ink-faint px-2 py-6 text-center text-[12px]">
            还没有对话，直接在右侧提问即可开始。
          </p>
        ) : (
          <ul className="space-y-0.5">
            {conversations.map((conversation) => {
              const active = conversation.id === activeId;
              return (
                <li key={conversation.id} className="group relative">
                  <button
                    type="button"
                    onClick={() => onSelect(conversation.id)}
                    className={cn(
                      "flex w-full items-center gap-2 rounded-lg px-2.5 py-2 pr-14 text-left text-[12.5px] transition-colors",
                      active
                        ? "bg-brand-soft text-brand font-medium"
                        : "text-ink-muted hover:bg-surface-raised hover:text-ink",
                    )}
                  >
                    <span className="min-w-0 flex-1 truncate">{conversation.title}</span>
                    <span className="text-ink-faint shrink-0 text-[10px] tabular-nums">
                      {conversation.messageCount}
                    </span>
                  </button>

                  <div className="absolute top-1/2 right-1 flex -translate-y-1/2 gap-0.5 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100">
                    <IconButton
                      label={`重命名对话 ${conversation.title}`}
                      className="size-6"
                      onClick={(event) => {
                        event.stopPropagation();
                        setTitle(conversation.title);
                        setRenameTarget(conversation);
                      }}
                    >
                      <Pencil size={12} />
                    </IconButton>
                    <IconButton
                      label={`删除对话 ${conversation.title}`}
                      className="size-6"
                      onClick={(event) => {
                        event.stopPropagation();
                        setDeleteTarget(conversation);
                      }}
                    >
                      <Trash2 size={12} />
                    </IconButton>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>

      {renameTarget ? (
        <Dialog
          open
          title="重命名对话"
          onClose={() => setRenameTarget(null)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setRenameTarget(null)} disabled={busy}>
                取消
              </Button>
              <Button
                disabled={busy}
                onClick={async () => {
                  const trimmed = title.trim();
                  if (!trimmed) return;
                  setBusy(true);
                  await onRename(renameTarget.id, trimmed);
                  setBusy(false);
                  setRenameTarget(null);
                }}
              >
                保存
              </Button>
            </>
          }
        >
          <Input
            label="标题"
            value={title}
            autoFocus
            maxLength={80}
            onChange={(event) => setTitle(event.currentTarget.value)}
          />
        </Dialog>
      ) : null}

      {deleteTarget ? (
        <Dialog
          open
          title="删除对话？"
          description={`「${deleteTarget.title}」的所有消息都会从本机删除，且无法恢复。`}
          onClose={() => setDeleteTarget(null)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setDeleteTarget(null)} disabled={busy}>
                取消
              </Button>
              <Button
                variant="danger"
                disabled={busy}
                onClick={async () => {
                  setBusy(true);
                  await onDelete(deleteTarget.id);
                  setBusy(false);
                  setDeleteTarget(null);
                }}
              >
                删除
              </Button>
            </>
          }
        />
      ) : null}
    </aside>
  );
}
