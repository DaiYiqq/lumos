import { Settings } from "lucide-react";

import { EmptyState } from "@/components/ui/EmptyState";

/** 设置一级页面：AI Provider、嵌入模型、语言与数据目录。 */
export function SettingsView() {
  return (
    <EmptyState
      icon={<Settings size={22} aria-hidden="true" />}
      title="设置"
      description="API Key、Ollama 地址、嵌入模型与界面语言将在这里配置。所有凭据保存在系统钥匙串里，不会写入配置文件，也不会进入日志。"
    />
  );
}
