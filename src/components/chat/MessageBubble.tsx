import { Loader2, Sparkles } from "lucide-react";

import { Markdown } from "@/components/chat/Markdown";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import type { DisplayMessage } from "@/stores/chat";

interface MessageBubbleProps {
  /** 消息内容。 */
  message: DisplayMessage;
  /** 是否正在流式输出（最后一条助手消息）。 */
  streaming: boolean;
  /** 点击引用。 */
  onCitation: (index: number) => void;
}

/** 单条消息气泡。 */
export function MessageBubble({ message, streaming, onCitation }: MessageBubbleProps) {
  const isUser = message.role === "user";
  const t = useT();

  return (
    <div className={cn("flex gap-3", isUser ? "flex-row-reverse" : "flex-row")}>
      <div
        className={cn(
          "mt-0.5 grid size-7 shrink-0 place-items-center rounded-full",
          isUser ? "bg-brand text-white" : "bg-brand-soft text-brand",
        )}
        aria-hidden="true"
      >
        {isUser ? (
          <span className="text-[11px] font-semibold">{t("我")}</span>
        ) : (
          <Sparkles size={14} />
        )}
      </div>

      <div
        className={cn(
          "max-w-[85%] min-w-0 rounded-xl px-3.5 py-2.5",
          isUser ? "bg-brand text-white" : "border-line bg-surface text-ink border shadow-sm",
        )}
      >
        {isUser ? (
          <p className="text-[13.5px] leading-relaxed whitespace-pre-wrap">{message.content}</p>
        ) : message.content ? (
          <Markdown content={message.content} onCitation={onCitation} />
        ) : (
          <div className="text-ink-faint flex items-center gap-2 text-[13px]">
            <Loader2 size={14} className="animate-spin" aria-hidden="true" />
            {t("正在检索并生成回答…")}
          </div>
        )}

        {streaming && message.content ? (
          <span className="bg-brand ml-0.5 inline-block h-3.5 w-1 animate-pulse rounded-sm align-middle" />
        ) : null}
      </div>
    </div>
  );
}
