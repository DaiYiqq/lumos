import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface EmptyStateProps {
  /** 顶部图标。 */
  icon: ReactNode;
  /** 主标题。 */
  title: string;
  /** 一句人话说明。 */
  description: string;
  /** 可选操作区。 */
  children?: ReactNode;
  /** 附加样式类。 */
  className?: string;
}

/** 空状态：所有一级页面共享的统一视觉。 */
export function EmptyState({ icon, title, description, children, className }: EmptyStateProps) {
  return (
    <div
      className={cn(
        "lumos-fade-up mx-auto flex max-w-xl flex-col items-center px-6 py-16 text-center",
        className,
      )}
    >
      <div className="border-line bg-surface text-brand mb-6 grid size-14 place-items-center rounded-2xl border shadow-sm">
        {icon}
      </div>
      <h1 className="text-ink text-xl font-semibold tracking-tight">{title}</h1>
      <p className="text-ink-muted mt-2 max-w-md text-sm leading-relaxed">{description}</p>
      {children ? <div className="mt-7 flex items-center gap-3">{children}</div> : null}
    </div>
  );
}
