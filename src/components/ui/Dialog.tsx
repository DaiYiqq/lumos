import { X } from "lucide-react";
import { useEffect } from "react";
import type { ReactNode } from "react";
import { createPortal } from "react-dom";

import { cn } from "@/lib/utils";

interface DialogProps {
  /** 是否显示。 */
  open: boolean;
  /** 标题。 */
  title: string;
  /** 可选说明文字。 */
  description?: string;
  /** 关闭回调（点击遮罩、按 Esc 或右上角关闭按钮触发）。 */
  onClose: () => void;
  /** 主体内容。 */
  children?: ReactNode;
  /** 底部操作区。 */
  footer?: ReactNode;
  /** 面板附加样式。 */
  className?: string;
}

/**
 * 极简模态对话框：不引入组件库，遮罩 + 面板 + Esc 关闭。
 *
 * 打开时把焦点移到面板上，保证键盘用户不会困在背景内容里。
 */
export function Dialog({
  open,
  title,
  description,
  onClose,
  children,
  footer,
  className,
}: DialogProps) {
  useEffect(() => {
    if (!open) return;

    const handleKeyDown = (event: KeyboardEvent): void => {
      if (event.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose]);

  if (!open) return null;

  return createPortal(
    <div className="fixed inset-0 z-50 flex items-center justify-center p-6">
      <div
        className="absolute inset-0 bg-black/45 backdrop-blur-[2px]"
        aria-hidden="true"
        onClick={onClose}
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-label={title}
        tabIndex={-1}
        className={cn(
          "bg-surface border-line lumos-fade-up relative w-full max-w-md rounded-xl border p-5 text-left shadow-2xl",
          className,
        )}
      >
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-ink text-base font-semibold">{title}</h2>
            {description ? (
              <p className="text-ink-muted mt-1 text-[13px] leading-relaxed">{description}</p>
            ) : null}
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="关闭"
            className="text-ink-faint hover:bg-surface-raised hover:text-ink -mt-1 -mr-1 rounded-md p-1.5 transition-colors"
          >
            <X size={16} />
          </button>
        </div>

        {children ? <div className="mt-4">{children}</div> : null}
        {footer ? <div className="mt-6 flex justify-end gap-2">{footer}</div> : null}
      </div>
    </div>,
    document.body,
  );
}
