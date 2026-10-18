import { create } from "zustand";

import * as tauri from "@/lib/tauri";
import type { ChatEvent, Citation, ConversationSummary, MessageRole, StoredMessage } from "@/types";

/** 界面上展示的一条消息。 */
export interface DisplayMessage {
  /** 本地/后端消息 ID。 */
  id: string;
  /** 角色。 */
  role: MessageRole;
  /** 正文。 */
  content: string;
  /** 引用（助手消息）。 */
  citations: Citation[] | null;
}

interface ChatState {
  /** 当前知识库。 */
  libraryId: string | null;
  /** 会话列表。 */
  conversations: ConversationSummary[];
  /** 当前会话 ID；null 表示尚未创建（发送第一条消息时创建）。 */
  activeId: string | null;
  /** 当前消息列表。 */
  messages: DisplayMessage[];
  /** 当前回答的引用。 */
  citations: Citation[];
  /** 引用面板中选中的编号。 */
  selectedCitation: number | null;
  /** 是否正在流式生成。 */
  streaming: boolean;
  /** 当前 Provider 标识。 */
  provider: string | null;
  /** Provider 展示名。 */
  providerLabel: string | null;
  /** 提示信息（降级等）。 */
  notice: string | null;
  /** 错误信息。 */
  error: string | null;
  /** 会话列表是否加载中。 */
  loading: boolean;

  /** 切换知识库并加载会话列表。 */
  selectLibrary: (libraryId: string) => Promise<void>;
  /** 刷新会话列表。 */
  refreshConversations: () => Promise<void>;
  /** 开始新对话（不创建空会话，发送时才创建）。 */
  startNew: () => void;
  /** 打开历史会话。 */
  openConversation: (id: string) => Promise<void>;
  /** 重命名会话。 */
  renameConversation: (id: string, title: string) => Promise<void>;
  /** 删除会话。 */
  deleteConversation: (id: string) => Promise<void>;
  /** 发送问题并流式接收回答。 */
  send: (question: string) => Promise<void>;
  /** 选中/取消选中引用。 */
  selectCitation: (index: number | null) => void;
  /** 清除错误。 */
  clearError: () => void;
}

/** 把未知错误转成一句可展示的话。 */
function describeError(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return "操作失败，请重试";
}

/** 从消息里解析引用 JSON。 */
function parseCitations(message: StoredMessage): Citation[] | null {
  if (!message.citationsJson) return null;
  try {
    const parsed: unknown = JSON.parse(message.citationsJson);
    return Array.isArray(parsed) ? (parsed as Citation[]) : null;
  } catch {
    return null;
  }
}

/** 把后端消息转成界面消息。 */
function toDisplayMessage(message: StoredMessage): DisplayMessage {
  return {
    id: message.id,
    role: message.role,
    content: message.content,
    citations: parseCitations(message),
  };
}

/** 问答状态。 */
export const useChatStore = create<ChatState>((set, get) => ({
  libraryId: null,
  conversations: [],
  activeId: null,
  messages: [],
  citations: [],
  selectedCitation: null,
  streaming: false,
  provider: null,
  providerLabel: null,
  notice: null,
  error: null,
  loading: false,

  selectLibrary: async (libraryId) => {
    set({
      libraryId,
      activeId: null,
      messages: [],
      citations: [],
      selectedCitation: null,
      notice: null,
      error: null,
      provider: null,
      providerLabel: null,
    });
    await get().refreshConversations();
  },

  refreshConversations: async () => {
    const libraryId = get().libraryId;
    if (!libraryId) return;
    set({ loading: true });
    try {
      const conversations = await tauri.listConversations(libraryId);
      set({ conversations, loading: false });
    } catch (error) {
      set({ loading: false, error: describeError(error) });
    }
  },

  startNew: () =>
    set({
      activeId: null,
      messages: [],
      citations: [],
      selectedCitation: null,
      notice: null,
      error: null,
      provider: null,
      providerLabel: null,
    }),

  openConversation: async (id) => {
    try {
      const detail = await tauri.getConversation(id);
      const messages = detail.messages.map(toDisplayMessage);
      const lastAssistant = [...messages].reverse().find((message) => message.role === "assistant");
      set({
        activeId: id,
        messages,
        citations: lastAssistant?.citations ?? [],
        selectedCitation: null,
        notice: null,
        error: null,
        provider: null,
        providerLabel: null,
      });
    } catch (error) {
      set({ error: describeError(error) });
    }
  },

  renameConversation: async (id, title) => {
    try {
      await tauri.renameConversation(id, title);
      await get().refreshConversations();
    } catch (error) {
      set({ error: describeError(error) });
    }
  },

  deleteConversation: async (id) => {
    try {
      await tauri.deleteConversation(id);
      if (get().activeId === id) get().startNew();
      await get().refreshConversations();
    } catch (error) {
      set({ error: describeError(error) });
    }
  },

  send: async (question) => {
    const libraryId = get().libraryId;
    const trimmed = question.trim();
    if (!libraryId || !trimmed || get().streaming) return;

    const history = get()
      .messages.filter((message) => message.role !== "system")
      .slice(-8)
      .map((message) => ({ role: message.role, content: message.content }));

    const userMessage: DisplayMessage = {
      id: `local-user-${Date.now()}`,
      role: "user",
      content: trimmed,
      citations: null,
    };
    const assistantId = `local-assistant-${Date.now()}`;
    const assistantMessage: DisplayMessage = {
      id: assistantId,
      role: "assistant",
      content: "",
      citations: null,
    };

    set((state) => ({
      messages: [...state.messages, userMessage, assistantMessage],
      streaming: true,
      notice: null,
      error: null,
      provider: null,
      providerLabel: null,
      citations: [],
      selectedCitation: null,
    }));

    const handleEvent = (event: ChatEvent): void => {
      switch (event.type) {
        case "conversation":
          set({ activeId: event.id });
          break;
        case "provider":
          set({ provider: event.provider, providerLabel: event.label });
          break;
        case "notice":
          set({ notice: event.message });
          break;
        case "citations":
          set({ citations: event.citations });
          break;
        case "delta":
          set((state) => ({
            messages: state.messages.map((message) =>
              message.id === assistantId
                ? { ...message, content: message.content + event.text }
                : message,
            ),
          }));
          break;
        case "done":
          set({ streaming: false });
          void get().refreshConversations();
          break;
        case "error":
          set({ streaming: false, error: event.message });
          break;
      }
    };

    try {
      await tauri.streamChat(libraryId, trimmed, get().activeId, history, handleEvent);
    } catch (error) {
      set({ streaming: false, error: describeError(error) });
    } finally {
      set({ streaming: false });
    }
  },

  selectCitation: (index) => set({ selectedCitation: index }),
  clearError: () => set({ error: null }),
}));
