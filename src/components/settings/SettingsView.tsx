import { AlertCircle, FolderOpen, KeyRound, Loader2, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import * as tauri from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { useAppInfoStore } from "@/stores/appInfo";
import { useThemeStore } from "@/stores/theme";
import type { ApiKeyStatus, EmbeddingStatus, ProviderStatus, ThemeMode } from "@/types";

const THEME_OPTIONS: Array<{ mode: ThemeMode; label: string }> = [
  { mode: "system", label: "跟随系统" },
  { mode: "light", label: "浅色" },
  { mode: "dark", label: "深色" },
];

/** 设置页：AI Provider、嵌入模型、外观与数据目录。 */
export function SettingsView() {
  const info = useAppInfoStore((state) => state.info);
  const themeMode = useThemeStore((state) => state.mode);
  const setThemeMode = useThemeStore((state) => state.setMode);

  const [apiKey, setApiKey] = useState("");
  const [keyStatus, setKeyStatus] = useState<ApiKeyStatus | null>(null);
  const [providers, setProviders] = useState<ProviderStatus[]>([]);
  const [embedding, setEmbedding] = useState<EmbeddingStatus | null>(null);
  const [ollamaUrl, setOllamaUrl] = useState("");
  const [ollamaModel, setOllamaModel] = useState("");
  const [deepseekModel, setDeepseekModel] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async (): Promise<void> => {
    setBusy(true);
    try {
      const [status, detection, embeddingState, url, ollama, deepseek] = await Promise.all([
        tauri.apiKeyStatus(),
        tauri.providersDetect(),
        tauri.embeddingStatus(),
        tauri.settingsGet("ollama_base_url"),
        tauri.settingsGet("ollama_model"),
        tauri.settingsGet("deepseek_model"),
      ]);
      setKeyStatus(status);
      setProviders(detection.statuses);
      setEmbedding(embeddingState);
      setOllamaUrl(url ?? "http://127.0.0.1:11434");
      setOllamaModel(ollama ?? "");
      setDeepseekModel(deepseek ?? "deepseek-chat");
    } catch (refreshError) {
      setError(describeError(refreshError));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    // 延后一拍调用，避免在 effect 内同步 setState 触发级联渲染（react-hooks 规则）。
    const timer = setTimeout(() => {
      void refresh();
    }, 0);
    return () => clearTimeout(timer);
  }, [refresh]);

  const saveApiKey = async (): Promise<void> => {
    if (!apiKey.trim()) {
      setError("请输入 API Key");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const status = await tauri.apiKeySet(apiKey.trim());
      setKeyStatus(status);
      setApiKey("");
      setNotice("API Key 已保存到系统凭据库");
      await refresh();
    } catch (saveError) {
      setError(describeError(saveError));
    } finally {
      setBusy(false);
    }
  };

  const clearApiKey = async (): Promise<void> => {
    setBusy(true);
    try {
      setKeyStatus(await tauri.apiKeyClear());
      setNotice("已删除系统凭据库中的 API Key");
      await refresh();
    } catch (clearError) {
      setError(describeError(clearError));
    } finally {
      setBusy(false);
    }
  };

  const saveSettings = async (): Promise<void> => {
    setBusy(true);
    setNotice(null);
    setError(null);
    try {
      await tauri.settingsSet("ollama_base_url", ollamaUrl.trim());
      await tauri.settingsSet("ollama_model", ollamaModel.trim());
      await tauri.settingsSet("deepseek_model", deepseekModel.trim());
      setNotice("设置已保存");
      await refresh();
    } catch (saveError) {
      setError(describeError(saveError));
    } finally {
      setBusy(false);
    }
  };

  const backfillEmbeddings = async (): Promise<void> => {
    setBusy(true);
    try {
      const written = await tauri.embeddingBackfill();
      setNotice(`已补写 ${written} 个分块的向量`);
      await refresh();
    } catch (backfillError) {
      setError(describeError(backfillError));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="mx-auto w-full max-w-3xl overflow-y-auto px-8 py-8">
      <header>
        <h1 className="text-ink text-xl font-semibold tracking-tight">设置</h1>
        <p className="text-ink-muted mt-1 text-[13px]">
          所有配置只保存在本机；API Key 存在系统凭据库，不会写入数据库或日志。
        </p>
      </header>

      {notice ? (
        <div className="border-line bg-surface-raised text-ink-muted mt-4 rounded-lg border px-3 py-2 text-[12.5px]">
          {notice}
        </div>
      ) : null}
      {error ? (
        <div
          role="alert"
          className="mt-4 flex items-start gap-2 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-[12.5px] text-red-600 dark:text-red-300"
        >
          <AlertCircle size={15} className="mt-0.5 shrink-0" aria-hidden="true" />
          <span className="flex-1">{error}</span>
        </div>
      ) : null}

      <section className="border-line bg-surface mt-5 rounded-xl border p-5">
        <div className="flex items-center gap-2">
          <KeyRound size={16} className="text-brand" aria-hidden="true" />
          <h2 className="text-ink text-[14px] font-semibold">DeepSeek API Key</h2>
          {busy ? (
            <Loader2 size={13} className="text-ink-faint animate-spin" aria-hidden="true" />
          ) : null}
        </div>
        <p className="text-ink-muted mt-1 text-[12px]">
          当前状态：
          {keyStatus?.configured
            ? keyStatus.source === "keyring"
              ? " 已配置（系统凭据库）"
              : " 已配置（环境变量）"
            : " 未配置，将使用本地 Ollama 或离线关键词模式"}
        </p>

        <div className="mt-3 flex flex-wrap items-end gap-2">
          <div className="min-w-64 flex-1">
            <Input
              label="API Key"
              type="password"
              autoComplete="off"
              placeholder="sk-..."
              value={apiKey}
              onChange={(event) => setApiKey(event.currentTarget.value)}
            />
          </div>
          <Button onClick={() => void saveApiKey()} disabled={busy}>
            保存到凭据库
          </Button>
          <Button variant="secondary" onClick={() => void clearApiKey()} disabled={busy}>
            删除
          </Button>
        </div>
      </section>

      <section className="border-line bg-surface mt-4 rounded-xl border p-5">
        <h2 className="text-ink text-[14px] font-semibold">模型服务</h2>
        <div className="mt-3 space-y-1.5">
          {providers.map((provider) => (
            <div
              key={provider.id}
              className="border-line flex items-center gap-2 rounded-lg border px-3 py-2 text-[12.5px]"
            >
              <span
                className={cn(
                  "size-1.5 rounded-full",
                  provider.available ? "bg-emerald-500" : "bg-ink-faint",
                )}
                aria-hidden="true"
              />
              <span className="text-ink w-20 font-medium capitalize">{provider.id}</span>
              <span className="text-ink-muted min-w-0 flex-1 truncate" title={provider.detail}>
                {provider.detail}
              </span>
              {provider.selected ? (
                <span className="text-brand bg-brand-soft rounded-full px-2 py-0.5 text-[10.5px]">
                  当前使用
                </span>
              ) : null}
            </div>
          ))}
        </div>

        <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2">
          <Input
            label="Ollama 地址"
            value={ollamaUrl}
            placeholder="http://127.0.0.1:11434"
            onChange={(event) => setOllamaUrl(event.currentTarget.value)}
          />
          <Input
            label="Ollama 模型"
            value={ollamaModel}
            placeholder="例如 qwen2.5:0.5b"
            onChange={(event) => setOllamaModel(event.currentTarget.value)}
          />
          <Input
            label="DeepSeek 模型"
            value={deepseekModel}
            placeholder="deepseek-chat"
            onChange={(event) => setDeepseekModel(event.currentTarget.value)}
          />
        </div>
        <div className="mt-3 flex gap-2">
          <Button onClick={() => void saveSettings()} disabled={busy}>
            保存设置
          </Button>
          <Button variant="secondary" onClick={() => void refresh()} disabled={busy}>
            <RefreshCw size={14} aria-hidden="true" />
            重新检测
          </Button>
        </div>
      </section>

      <section className="border-line bg-surface mt-4 rounded-xl border p-5">
        <h2 className="text-ink text-[14px] font-semibold">本地嵌入模型</h2>
        <p className="text-ink-muted mt-1 text-[12px]">
          {embedding
            ? embedding.state === "ready"
              ? `已就绪：${embedding.model}（${embedding.dimension} 维）`
              : embedding.state === "loading"
                ? "正在加载/下载模型（首次约 95MB）…"
                : `不可用：${embedding.message ?? "未知原因"}，检索将降级为关键词模式`
            : "读取中…"}
        </p>
        <div className="mt-3 flex gap-2">
          <Button variant="secondary" onClick={() => void backfillEmbeddings()} disabled={busy}>
            立即补写向量
          </Button>
          <Button variant="ghost" onClick={() => void refresh()} disabled={busy}>
            刷新状态
          </Button>
        </div>
      </section>

      <section className="border-line bg-surface mt-4 rounded-xl border p-5">
        <h2 className="text-ink text-[14px] font-semibold">外观</h2>
        <div className="mt-3 flex gap-2" role="group" aria-label="主题">
          {THEME_OPTIONS.map((option) => (
            <Button
              key={option.mode}
              size="sm"
              variant={themeMode === option.mode ? "primary" : "secondary"}
              onClick={() => setThemeMode(option.mode)}
            >
              {option.label}
            </Button>
          ))}
        </div>
      </section>

      <section className="border-line bg-surface mt-4 rounded-xl border p-5">
        <h2 className="text-ink text-[14px] font-semibold">数据目录</h2>
        <p className="text-ink-muted mt-1 font-mono text-[12px]" title={info?.dataDir}>
          {info?.dataDirDisplay ?? "读取中…"}
        </p>
        <p className="text-ink-faint mt-1 text-[11.5px]">
          数据库、日志与嵌入模型都保存在这里；卸载前可手动备份整个目录。
        </p>
        <div className="mt-3">
          <Button
            variant="secondary"
            disabled={!info || !tauri.isTauriRuntime()}
            onClick={() =>
              void tauri
                .openDataDir()
                .catch((openError: unknown) => setError(describeError(openError)))
            }
          >
            <FolderOpen size={14} aria-hidden="true" />
            打开数据目录
          </Button>
        </div>
      </section>
    </div>
  );
}

/** 把未知错误转成一句可展示的话。 */
function describeError(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return "操作失败，请重试";
}
