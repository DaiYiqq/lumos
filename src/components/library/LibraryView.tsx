import { Library } from "lucide-react";

import { Button } from "@/components/ui/Button";
import { EmptyState } from "@/components/ui/EmptyState";

/** 知识库一级页面：多库管理、导入来源、索引进度。 */
export function LibraryView() {
  return (
    <EmptyState
      icon={<Library size={22} aria-hidden="true" />}
      title="还没有知识库"
      description="创建一个知识库，然后把 PDF、EPUB、Word、Markdown 或网页拖进来。索引、向量与问答记录全部保存在本机。"
    >
      <Button disabled title="数据层接入后开放">
        新建知识库
      </Button>
      <Button variant="secondary" disabled title="数据层接入后开放">
        导入文件夹
      </Button>
    </EmptyState>
  );
}
