import { forwardRef, useId } from "react";
import type { InputHTMLAttributes } from "react";

import { cn } from "@/lib/utils";

/** Input 组件属性。 */
export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  /** 标签文字；传入后自动绑定 `htmlFor`。 */
  label?: string;
  /** 错误提示。 */
  error?: string;
  /** 辅助说明。 */
  hint?: string;
}

/** 带标签与错误提示的输入框。 */
export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { label, error, hint, className, id, ...props },
  ref,
) {
  const generatedId = useId();
  const inputId = id ?? generatedId;
  const describedBy = error ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined;

  return (
    <div className="flex flex-col gap-1.5">
      {label ? (
        <label htmlFor={inputId} className="text-ink-muted text-[13px] font-medium">
          {label}
        </label>
      ) : null}
      <input
        ref={ref}
        id={inputId}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy}
        className={cn(
          "border-line bg-surface-raised text-ink placeholder:text-ink-faint h-10 rounded-lg border px-3 text-sm",
          "focus:border-brand focus:outline-brand focus:outline-2 focus:outline-offset-1",
          "disabled:cursor-not-allowed disabled:opacity-60",
          error ? "border-red-500 focus:border-red-500 focus:outline-red-500" : null,
          className,
        )}
        {...props}
      />
      {error ? (
        <p id={`${inputId}-error`} className="text-[12px] text-red-500">
          {error}
        </p>
      ) : hint ? (
        <p id={`${inputId}-hint`} className="text-ink-faint text-[12px]">
          {hint}
        </p>
      ) : null}
    </div>
  );
});
