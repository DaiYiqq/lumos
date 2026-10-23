import { Check, Copy, FileText, X } from "lucide-react";
import { useState } from "react";

import { useT } from "@/i18n";
import { copyToClipboard } from "@/lib/clipboard";
import { cn } from "@/lib/utils";
import type { Citation } from "@/types";

interface CitationPanelProps {
  /** 引用列表。 */
  citations: Citation[];
  /** 当前选中的引用编号。 */
  selected: number | null;
  /** 选中引用。 */
  onSelect: (index: number) => void;
  /** 关闭面板。 */
  onClose: () => void;
}

/** 右侧引用面板：展示原文片段与位置。 */
export function CitationPanel({ citations, selected, onSelect, onClose }: CitationPanelProps) {
  const t = useT();
  const [copied, setCopied] = useState(false);
  const activeCitation = citations.find((citation) => citation.index === selected) ?? citations[0];

  const copyCitation = async (): Promise<void> => {
    if (!activeCitation) return;
    const location = [
      activeCitation.pageNo ? t("第 {page} 页", { page: activeCitation.pageNo }) : null,
      activeCitation.sectionPath,
    ]
      .filter(Boolean)
      .join(" · ");
    const text = [
      `[${activeCitation.index}] ${activeCitation.documentTitle}`,
      location ? `位置：${location}` : null,
      activeCitation.content,
      `来源：${activeCitation.documentPath}`,
    ]
      .filter(Boolean)
      .join("\n");
    if (await copyToClipboard(text)) {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    }
  };

  return (
    <aside
      aria-label={t("引用面板")}
      className="border-line bg-surface flex w-80 shrink-0 flex-col border-l"
    >
      <header className="border-line flex h-11 shrink-0 items-center justify-between border-b px-3">
        <span className="text-ink text-[13px] font-semibold">{t("引用来源")}</span>
        <div className="flex items-center gap-0.5">
          <button
            type="button"
            onClick={() => void copyCitation()}
            disabled={citations.length === 0}
            aria-label={t("复制引用")}
            className="text-ink-faint hover:bg-surface-raised hover:text-ink rounded-md p-1 transition-colors disabled:opacity-40"
          >
            {copied ? <Check size={14} /> : <Copy size={14} />}
          </button>
          <button
            type="button"
            onClick={onClose}
            aria-label={t("关闭引用面板")}
            className="text-ink-faint hover:bg-surface-raised hover:text-ink rounded-md p-1 transition-colors"
          >
            <X size={14} />
          </button>
        </div>
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto p-2">
        {citations.length === 0 ? (
          <p className="text-ink-faint px-2 py-6 text-center text-[12px] leading-relaxed">
            {t("回答中的 [1]、[2] 会在这里显示原文片段。")}
            <br />
            {t("点击任意编号即可跳转对照。")}
          </p>
        ) : (
          <ul className="space-y-1.5">
            {citations.map((citation) => {
              const active = citation.index === selected;
              return (
                <li key={citation.index}>
                  <button
                    type="button"
                    onClick={() => onSelect(citation.index)}
                    className={cn(
                      "w-full rounded-lg border p-2.5 text-left transition-colors",
                      active ? "border-brand bg-brand-soft" : "border-line hover:bg-surface-raised",
                    )}
                  >
                    <div className="flex items-center gap-1.5">
                      <span className="bg-brand inline-flex h-4 min-w-4 items-center justify-center rounded px-1 text-[10px] font-semibold text-white">
                        {citation.index}
                      </span>
                      <FileText size={12} className="text-ink-faint shrink-0" aria-hidden="true" />
                      <span className="text-ink truncate text-[12px] font-medium">
                        {citation.documentTitle}
                      </span>
                    </div>
                    <p
                      className="text-ink-faint mt-1 truncate text-[10.5px]"
                      title={citation.documentPath}
                    >
                      {[
                        citation.pageNo ? t("第 {page} 页", { page: citation.pageNo }) : null,
                        citation.sectionPath,
                        citation.documentPath,
                      ]
                        .filter(Boolean)
                        .join(" · ")}
                    </p>
                    <p className="text-ink-muted mt-1.5 line-clamp-4 text-[11.5px] leading-relaxed whitespace-pre-wrap">
                      {citation.content}
                    </p>
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </aside>
  );
}
