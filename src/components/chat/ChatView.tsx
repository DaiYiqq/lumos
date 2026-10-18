import { MessagesSquare } from "lucide-react";

import { Button } from "@/components/ui/Button";
import { EmptyState } from "@/components/ui/EmptyState";

/** 问答一级页面：流式回答与引用溯源面板。 */
export function ChatView() {
  return (
    <EmptyState
      icon={<MessagesSquare size={22} aria-hidden="true" />}
      title="从一个问题开始"
      description="选择知识库后提问，回答里的 [1] [2] 可以点击，右侧会高亮对应的原文片段。没有 API Key 时，Lumos 会自动降级为离线关键词检索。"
    >
      <Button disabled title="检索层接入后开放">
        开始提问
      </Button>
    </EmptyState>
  );
}
