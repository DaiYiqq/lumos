import { PanelRight, Sparkles } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { CitationPanel } from "@/components/chat/CitationPanel";
import { Composer } from "@/components/chat/Composer";
import { ConversationList } from "@/components/chat/ConversationList";
import { MessageBubble } from "@/components/chat/MessageBubble";
import { EmptyState } from "@/components/ui/EmptyState";
import { IconButton } from "@/components/ui/IconButton";
import { cn } from "@/lib/utils";
import { useChatStore } from "@/stores/chat";
import { useLibrariesStore } from "@/stores/libraries";
import { useUiStore } from "@/stores/ui";
import { Button } from "@/components/ui/Button";

/** 三个核心场景的示例问题。 */
const STARTER_QUESTIONS = [
  "有哪些工作用了对比学习做医学图像分割？",
  "合同里对违约金上限是怎么约定的？",
  "我去年夏天为什么决定换工作？",
];

/** 问答一级页面：会话列表 + 消息流 + 引用面板。 */
export function ChatView() {
  const libraries = useLibrariesStore((state) => state.items);
  const librariesLoaded = useLibrariesStore((state) => state.loaded);
  const loadLibraries = useLibrariesStore((state) => state.load);
  const selectedLibraryId = useUiStore((state) => state.selectedLibraryId);
  const setSelectedLibraryId = useUiStore((state) => state.setSelectedLibraryId);
  const setActiveView = useUiStore((state) => state.setActiveView);

  const conversations = useChatStore((state) => state.conversations);
  const activeId = useChatStore((state) => state.activeId);
  const messages = useChatStore((state) => state.messages);
  const citations = useChatStore((state) => state.citations);
  const selectedCitation = useChatStore((state) => state.selectedCitation);
  const streaming = useChatStore((state) => state.streaming);
  const provider = useChatStore((state) => state.provider);
  const providerLabel = useChatStore((state) => state.providerLabel);
  const notice = useChatStore((state) => state.notice);
  const error = useChatStore((state) => state.error);
  const loadingConversations = useChatStore((state) => state.loading);
  const selectLibrary = useChatStore((state) => state.selectLibrary);
  const startNew = useChatStore((state) => state.startNew);
  const openConversation = useChatStore((state) => state.openConversation);
  const renameConversation = useChatStore((state) => state.renameConversation);
  const deleteConversation = useChatStore((state) => state.deleteConversation);
  const send = useChatStore((state) => state.send);
  const selectCitation = useChatStore((state) => state.selectCitation);
  const clearError = useChatStore((state) => state.clearError);

  const [showCitations, setShowCitations] = useState(true);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!librariesLoaded) void loadLibraries();
  }, [librariesLoaded, loadLibraries]);

  useEffect(() => {
    if (!selectedLibraryId && libraries.length > 0) {
      setSelectedLibraryId(libraries[0]?.id ?? null);
    }
  }, [selectedLibraryId, libraries, setSelectedLibraryId]);

  useEffect(() => {
    if (selectedLibraryId) void selectLibrary(selectedLibraryId);
  }, [selectedLibraryId, selectLibrary]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView?.({ behavior: streaming ? "auto" : "smooth" });
  }, [messages, streaming]);

  const currentLibrary = libraries.find((library) => library.id === selectedLibraryId) ?? null;

  if (librariesLoaded && libraries.length === 0) {
    return (
      <EmptyState
        icon={<Sparkles size={22} aria-hidden="true" />}
        title="先创建一个知识库"
        description="问答基于你导入的文档。先创建知识库并导入 PDF、Word、Markdown 等文件，再回来提问。"
      >
        <Button onClick={() => setActiveView("library")}>去创建知识库</Button>
      </EmptyState>
    );
  }

  return (
    <div className="flex h-full min-h-0">
      <ConversationList
        conversations={conversations}
        activeId={activeId}
        loading={loadingConversations}
        onSelect={(id) => void openConversation(id)}
        onNew={startNew}
        onRename={renameConversation}
        onDelete={deleteConversation}
      />

      <section className="flex min-h-0 min-w-0 flex-1 flex-col">
        <header className="border-line flex h-11 shrink-0 items-center gap-3 border-b px-4">
          <select
            value={selectedLibraryId ?? ""}
            onChange={(event) => setSelectedLibraryId(event.currentTarget.value || null)}
            aria-label="选择知识库"
            className="border-line bg-surface text-ink focus:border-brand rounded-md border px-2 py-1 text-[12.5px] outline-none"
          >
            {libraries.map((library) => (
              <option key={library.id} value={library.id}>
                {library.name}
              </option>
            ))}
          </select>
          {currentLibrary ? (
            <span className="text-ink-faint hidden text-[11px] sm:inline">
              {currentLibrary.documentCount} 篇文档 · {currentLibrary.chunkCount} 个分块
            </span>
          ) : null}

          <div className="ml-auto flex items-center gap-1.5">
            {providerLabel ? (
              <span
                className={cn(
                  "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium",
                  provider === "deepseek" && "bg-brand-soft text-brand",
                  provider === "ollama" &&
                    "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400",
                  provider === "none" && "bg-amber-500/10 text-amber-700 dark:text-amber-400",
                )}
                title={notice ?? undefined}
              >
                <span
                  className={cn(
                    "size-1.5 rounded-full",
                    provider === "deepseek" && "bg-brand",
                    provider === "ollama" && "bg-emerald-500",
                    provider === "none" && "bg-amber-500",
                  )}
                  aria-hidden="true"
                />
                {providerLabel}
              </span>
            ) : null}
            <IconButton
              label={showCitations ? "隐藏引用面板" : "显示引用面板"}
              onClick={() => setShowCitations((value) => !value)}
              className="border-line border"
            >
              <PanelRight size={15} />
            </IconButton>
          </div>
        </header>

        {notice ? (
          <div className="border-line bg-surface-raised text-ink-muted shrink-0 border-b px-4 py-1.5 text-[11.5px]">
            {notice}
          </div>
        ) : null}

        {error ? (
          <div
            role="alert"
            className="flex shrink-0 items-center gap-2 border-b border-red-500/30 bg-red-500/10 px-4 py-2 text-[12.5px] text-red-600 dark:text-red-300"
          >
            <span className="flex-1">{error}</span>
            <button type="button" onClick={clearError} className="text-[11px] underline">
              知道了
            </button>
          </div>
        ) : null}

        <div className="min-h-0 flex-1 overflow-y-auto px-4 py-5">
          {messages.length === 0 ? (
            <div className="mx-auto max-w-2xl">
              <div className="text-center">
                <div className="bg-brand-soft text-brand mx-auto grid size-11 place-items-center rounded-xl">
                  <Sparkles size={20} aria-hidden="true" />
                </div>
                <h2 className="text-ink mt-3 text-[15px] font-semibold">
                  基于「{currentLibrary?.name ?? "知识库"}」提问
                </h2>
                <p className="text-ink-muted mt-1 text-[12.5px]">
                  回答会标注引用编号，点击即可查看原文片段。
                </p>
              </div>
              <div className="mt-5 space-y-2">
                <p className="text-ink-faint text-[11px]">试试这样问：</p>
                {STARTER_QUESTIONS.map((question) => (
                  <button
                    key={question}
                    type="button"
                    onClick={() => void send(question)}
                    disabled={streaming}
                    className="border-line bg-surface text-ink-muted hover:border-brand hover:text-ink w-full rounded-lg border px-3 py-2 text-left text-[12.5px] transition-colors disabled:opacity-50"
                  >
                    {question}
                  </button>
                ))}
              </div>
            </div>
          ) : (
            <div className="mx-auto flex max-w-3xl flex-col gap-4">
              {messages.map((message) => (
                <MessageBubble
                  key={message.id}
                  message={message}
                  streaming={
                    streaming && message.role === "assistant" && message === messages.at(-1)
                  }
                  onCitation={(index) => {
                    selectCitation(index);
                    setShowCitations(true);
                  }}
                />
              ))}
              <div ref={bottomRef} />
            </div>
          )}
        </div>

        <Composer streaming={streaming} disabled={!currentLibrary} onSend={send} />
      </section>

      {showCitations ? (
        <CitationPanel
          citations={citations}
          selected={selectedCitation}
          onSelect={(index) => selectCitation(index)}
          onClose={() => setShowCitations(false)}
        />
      ) : null}
    </div>
  );
}
