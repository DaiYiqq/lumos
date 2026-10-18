import { forwardRef } from "react";
import type { ButtonHTMLAttributes } from "react";

import { cn } from "@/lib/utils";

/** 图标按钮属性。 */
export interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** 无障碍标签（必填，图标本身没有文字）。 */
  label: string;
}

/** 方形图标按钮：用于卡片上的轻量操作。 */
export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(function IconButton(
  { label, className, type = "button", children, ...props },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type}
      aria-label={label}
      title={label}
      className={cn(
        "text-ink-faint hover:bg-surface-raised hover:text-ink inline-flex size-7 items-center justify-center rounded-md transition-colors",
        "focus-visible:outline-brand focus-visible:outline-2 focus-visible:outline-offset-2",
        "disabled:pointer-events-none disabled:opacity-50",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
});
