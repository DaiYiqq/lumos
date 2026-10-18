import { useState } from "react";

import { Button } from "@/components/ui/Button";
import { Dialog } from "@/components/ui/Dialog";
import { Input } from "@/components/ui/Input";

interface LibraryNameDialogProps {
  /** 是否打开。 */
  open: boolean;
  /** 标题。 */
  title: string;
  /** 说明文字。 */
  description?: string;
  /** 初始名称（重命名时使用）。 */
  initialName?: string;
  /** 确认按钮文案。 */
  confirmLabel: string;
  /** 提交回调，返回 true 表示成功（对话框会自动关闭）。 */
  onSubmit: (name: string) => Promise<boolean>;
  /** 关闭回调。 */
  onClose: () => void;
}

const MAX_NAME_LENGTH = 80;

/**
 * 知识库命名对话框：新建与重命名共用。
 *
 * 由父组件在打开时挂载、关闭时卸载，因此表单状态天然在每次打开时重置，
 * 不需要在 effect 里 setState（也避免触发级联渲染）。
 */
export function LibraryNameDialog({
  open,
  title,
  description,
  initialName = "",
  confirmLabel,
  onSubmit,
  onClose,
}: LibraryNameDialogProps) {
  const [name, setName] = useState(initialName);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (): Promise<void> => {
    const trimmed = name.trim();
    if (!trimmed) {
      setError("请输入知识库名称");
      return;
    }
    if (trimmed.length > MAX_NAME_LENGTH) {
      setError(`名称不能超过 ${MAX_NAME_LENGTH} 个字符`);
      return;
    }

    setSubmitting(true);
    const success = await onSubmit(trimmed);
    setSubmitting(false);
    if (success) {
      onClose();
    }
  };

  return (
    <Dialog
      open={open}
      title={title}
      description={description}
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose} disabled={submitting}>
            取消
          </Button>
          <Button onClick={() => void handleSubmit()} disabled={submitting}>
            {submitting ? "处理中…" : confirmLabel}
          </Button>
        </>
      }
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void handleSubmit();
        }}
      >
        <Input
          label="知识库名称"
          value={name}
          autoFocus
          maxLength={MAX_NAME_LENGTH}
          placeholder="例如：合同条款、毕业论文、读书笔记"
          error={error ?? undefined}
          onChange={(event) => {
            setName(event.currentTarget.value);
            if (error) setError(null);
          }}
        />
        <button type="submit" className="hidden" aria-hidden="true" tabIndex={-1} />
      </form>
    </Dialog>
  );
}
