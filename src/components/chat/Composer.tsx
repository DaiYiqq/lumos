import { Send } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/Button";

interface ComposerProps {
  /** 是否正在生成。 */
  streaming: boolean;
  /** 是否禁用（没有知识库等）。 */
  disabled: boolean;
  /** 发送问题。 */
  onSend: (question: string) => Promise<void>;
}

/** 底部输入框。 */
export function Composer({ streaming, disabled, onSend }: ComposerProps) {
  const [value, setValue] = useState("");

  const submit = async (): Promise<void> => {
    const question = value.trim();
    if (!question || streaming || disabled) return;
    setValue("");
    await onSend(question);
  };

  return (
    <div className="border-line bg-surface shrink-0 border-t px-4 py-3">
      <div className="border-line bg-surface-raised focus-within:border-brand mx-auto flex max-w-3xl items-end gap-2 rounded-xl border px-3 py-2 transition-colors">
        <textarea
          value={value}
          rows={1}
          autoFocus
          disabled={disabled || streaming}
          placeholder={
            disabled
              ? "先创建一个知识库并导入文档"
              : "基于你的文档提问，Enter 发送，Shift+Enter 换行"
          }
          aria-label="提问输入框"
          className="text-ink placeholder:text-ink-faint max-h-40 min-h-6 flex-1 resize-none bg-transparent py-1 text-[13.5px] leading-relaxed outline-none disabled:cursor-not-allowed"
          onChange={(event) => setValue(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              void submit();
            }
          }}
        />
        <Button
          size="sm"
          className="mb-0.5"
          disabled={disabled || streaming || value.trim().length === 0}
          onClick={() => void submit()}
          aria-label="发送问题"
        >
          <Send size={14} aria-hidden="true" />
          {streaming ? "生成中" : "发送"}
        </Button>
      </div>
      <p className="text-ink-faint mx-auto mt-1.5 max-w-3xl text-[10.5px]">
        回答只依据你导入的文档生成；没有引用依据时会明确说明。
      </p>
    </div>
  );
}
