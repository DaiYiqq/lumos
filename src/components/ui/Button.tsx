import { forwardRef } from "react";
import type { ButtonHTMLAttributes } from "react";

import { cn } from "@/lib/utils";

type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
type ButtonSize = "sm" | "md";

/** Button 组件属性。 */
export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** 视觉层级，默认 `primary`。 */
  variant?: ButtonVariant;
  /** 尺寸，默认 `md`。 */
  size?: ButtonSize;
}

const VARIANT_CLASSES: Record<ButtonVariant, string> = {
  primary:
    "bg-brand text-white shadow-sm shadow-brand/25 hover:bg-brand-strong focus-visible:outline-brand",
  secondary:
    "border border-line bg-surface text-ink hover:bg-surface-raised focus-visible:outline-brand",
  ghost: "text-ink-muted hover:bg-surface-raised hover:text-ink focus-visible:outline-brand",
  danger:
    "bg-red-600 text-white shadow-sm shadow-red-600/25 hover:bg-red-700 focus-visible:outline-red-600",
};

const SIZE_CLASSES: Record<ButtonSize, string> = {
  sm: "h-8 gap-1.5 px-3 text-[13px]",
  md: "h-10 gap-2 px-4 text-sm",
};

/**
 * 极简按钮：项目不引入重型 UI 库，交互组件均在此目录自研。
 */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = "primary", size = "md", className, type = "button", ...props },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type}
      className={cn(
        "inline-flex items-center justify-center rounded-lg font-medium transition-colors select-none",
        "focus-visible:outline-2 focus-visible:outline-offset-2",
        "disabled:pointer-events-none disabled:opacity-50",
        VARIANT_CLASSES[variant],
        SIZE_CLASSES[size],
        className,
      )}
      {...props}
    />
  );
});
