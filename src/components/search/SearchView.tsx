import { FileText, Loader2, Search } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/Button";
import { EmptyState } from "@/components/ui/EmptyState";
import { useT } from "@/i18n";
import { copyToClipboard } from "@/lib/clipboard";
import { searchChunks } from "@/lib/tauri";
import { formatBytes } from "@/lib/utils";
import { useLibrariesStore } from "@/stores/libraries";
import { useUiStore } from "@/stores/ui";
import type { SearchMode, SearchResponse } from "@/types";

const MODES: Array<{ value: SearchMode; label: string }> = [
  { value: "hybrid", label: "混合检索" },
  { value: "vector", label: "向量检索" },
  { value: "keyword", label: "关键词" },
];

function modeLabel(mode: string, t: (source: string) => string): string {
  if (mode === "vector") return t("向量检索");
  if (mode === "keyword") return t("关键词");
  return t("混合检索");
}

/** 独立搜索页：在知识库中检索原文片段，直接核对引用。 */
export function SearchView() {
  const libraries = useLibrariesStore((state) => state.items);
  const loaded = useLibrariesStore((state) => state.loaded);
  const load = useLibrariesStore((state) => state.load);
  const selectedLibraryId = useUiStore((state) => state.selectedLibraryId);
  const setSelectedLibraryId = useUiStore((state) => state.setSelectedLibraryId);
  const t = useT();

  const [query, setQuery] = useState("");
  const [mode, setMode] = useState<SearchMode>("hybrid");
  const [response, setResponse] = useState<SearchResponse | null>(null);
  const [searching, setSearching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copiedChunk, setCopiedChunk] = useState<number | null>(null);

  useEffect(() => {
    if (!loaded) void load();
  }, [loaded, load]);

  useEffect(() => {
    if (!selectedLibraryId && libraries.length > 0) {
      setSelectedLibraryId(libraries[0]?.id ?? null);
    }
  }, [selectedLibraryId, libraries, setSelectedLibraryId]);

  const currentLibrary = libraries.find((library) => library.id === selectedLibraryId) ?? null;

  const runSearch = async (): Promise<void> => {
    const trimmed = query.trim();
    if (!currentLibrary || trimmed.length === 0 || searching) return;
    setSearching(true);
    setError(null);
    try {
      setResponse(await searchChunks(currentLibrary.id, trimmed, mode, 12));
    } catch (searchError) {
      setError(searchError instanceof Error ? searchError.message : String(searchError));
      setResponse(null);
    } finally {
      setSearching(false);
    }
  };

  if (loaded && libraries.length === 0) {
    return (
      <EmptyState
        icon={<Search size={22} aria-hidden="true" />}
        title={t("先创建一个知识库")}
        description={t(
          "搜索基于你导入的文档。先创建知识库并导入 PDF、Word、Markdown 等文件，再回来检索。",
        )}
      />
    );
  }

  return (
    <div className="mx-auto w-full max-w-4xl overflow-y-auto px-8 py-8">
      <header>
        <h1 className="text-ink text-xl font-semibold tracking-tight">{t("搜索")}</h1>
        <p className="text-ink-muted mt-1 text-[13px]">
          {t("在知识库中检索原文片段，结果可直接核对引用。")}
        </p>
      </header>

      <section className="border-line bg-surface mt-5 rounded-xl border p-4">
        <div className="flex flex-wrap items-center gap-2">
          <select
            value={selectedLibraryId ?? ""}
            onChange={(event) => setSelectedLibraryId(event.currentTarget.value || null)}
            aria-label={t("选择知识库")}
            className="border-line bg-surface text-ink focus:border-brand rounded-md border px-2 py-1.5 text-[12.5px] outline-none"
          >
            {libraries.map((library) => (
              <option key={library.id} value={library.id}>
                {library.name}
              </option>
            ))}
          </select>
          {currentLibrary ? (
            <span className="text-ink-faint text-[11px]">
              {t("{docs} 篇文档 · {chunks} 个分块", {
                docs: currentLibrary.documentCount,
                chunks: currentLibrary.chunkCount,
              })}
              {" · "}
              {formatBytes(currentLibrary.sizeBytes)}
            </span>
          ) : null}
          <div
            className="border-line ml-auto flex gap-1 rounded-lg border p-0.5"
            role="group"
            aria-label={t("检索模式")}
          >
            {MODES.map((item) => (
              <button
                key={item.value}
                type="button"
                onClick={() => setMode(item.value)}
                aria-pressed={mode === item.value}
                className={
                  mode === item.value
                    ? "bg-brand-soft text-brand rounded-md px-2 py-1 text-[11.5px] font-medium"
                    : "text-ink-muted hover:text-ink rounded-md px-2 py-1 text-[11.5px]"
                }
              >
                {t(item.label)}
              </button>
            ))}
          </div>
        </div>

        <form
          className="mt-3 flex gap-2"
          onSubmit={(event) => {
            event.preventDefault();
            void runSearch();
          }}
        >
          <input
            value={query}
            autoFocus
            placeholder={t("输入关键词检索文档…")}
            aria-label={t("搜索关键词")}
            onChange={(event) => setQuery(event.currentTarget.value)}
            className="border-line bg-surface-raised text-ink placeholder:text-ink-faint focus:border-brand h-9 min-w-0 flex-1 rounded-lg border px-3 text-[13px] outline-none"
          />
          <Button
            type="submit"
            disabled={searching || !currentLibrary || query.trim().length === 0}
          >
            {searching ? (
              <Loader2 size={14} className="animate-spin" aria-hidden="true" />
            ) : (
              <Search size={14} aria-hidden="true" />
            )}
            {t("搜索文档")}
          </Button>
        </form>
      </section>

      {error ? (
        <div
          role="alert"
          className="mt-4 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-[12.5px] text-red-600 dark:text-red-300"
        >
          {error}
        </div>
      ) : null}

      {response ? (
        <section className="mt-5" aria-live="polite">
          <p className="text-ink-faint text-[11px]">
            {t("{count} 条结果", { count: response.hits.length })}
            {" · "}
            {modeLabel(response.mode, t)}
            {response.requestedMode !== response.mode ? t("（向量模型不可用，已回退）") : null}
          </p>
          {response.hits.length === 0 ? (
            <EmptyState
              className="py-16"
              icon={<Search size={20} aria-hidden="true" />}
              title={t("没有找到相关片段")}
              description={t("换个关键词，或切换检索模式再试试。")}
            />
          ) : (
            <ul className="mt-3 space-y-2">
              {response.hits.map((hit, index) => (
                <li key={hit.chunkId} className="border-line bg-surface rounded-xl border p-3.5">
                  <div className="flex items-start gap-2">
                    <span className="bg-brand inline-flex h-5 min-w-5 items-center justify-center rounded px-1 text-[10px] font-semibold text-white">
                      {index + 1}
                    </span>
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
                        <span className="text-ink text-[13px] font-medium">
                          {hit.documentTitle}
                        </span>
                        <span className="text-ink-faint text-[10.5px]">
                          {[
                            hit.pageNo ? t("第 {page} 页", { page: hit.pageNo }) : null,
                            hit.sectionPath,
                            t("分数 {score}", { score: hit.score.toFixed(3) }),
                          ]
                            .filter(Boolean)
                            .join(" · ")}
                        </span>
                      </div>
                      <p className="text-ink-muted mt-1.5 text-[12px] leading-relaxed whitespace-pre-wrap">
                        {hit.content}
                      </p>
                      <div className="mt-2 flex items-center gap-2">
                        <span
                          className="text-ink-faint min-w-0 flex-1 truncate text-[10px]"
                          title={hit.documentPath}
                        >
                          {hit.documentPath}
                        </span>
                        <button
                          type="button"
                          onClick={async () => {
                            const text = `${hit.documentTitle}\n${hit.content}\n来源：${hit.documentPath}`;
                            if (await copyToClipboard(text)) {
                              setCopiedChunk(hit.chunkId);
                              window.setTimeout(() => setCopiedChunk(null), 1500);
                            }
                          }}
                          className="text-ink-faint hover:text-brand inline-flex items-center gap-1 text-[10.5px]"
                        >
                          <FileText size={11} aria-hidden="true" />
                          {copiedChunk === hit.chunkId ? t("已复制") : t("复制")}
                        </button>
                      </div>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </section>
      ) : null}
    </div>
  );
}
