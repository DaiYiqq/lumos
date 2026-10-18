import { create } from "zustand";

import { translate, useI18nStore } from "@/i18n";
import * as tauri from "@/lib/tauri";
import type { LibrarySummary } from "@/types";

interface LibrariesState {
  /** 知识库列表。 */
  items: LibrarySummary[];
  /** 是否已经成功加载过一次（用于区分「加载中」与「确实为空」）。 */
  loaded: boolean;
  /** 是否正在加载列表。 */
  loading: boolean;
  /** 最近一次操作的错误文案。 */
  error: string | null;
  /** 拉取列表。 */
  load: () => Promise<void>;
  /** 创建知识库，返回是否成功。 */
  create: (name: string) => Promise<boolean>;
  /** 重命名知识库，返回是否成功。 */
  rename: (id: string, name: string) => Promise<boolean>;
  /** 删除知识库，返回是否成功。 */
  remove: (id: string) => Promise<boolean>;
  /** 清除错误提示。 */
  clearError: () => void;
}

/** 把未知错误转成一句可展示的话。 */
function describeError(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return "操作失败，请重试";
}

/** 知识库列表状态。 */
export const useLibrariesStore = create<LibrariesState>((set, get) => ({
  items: [],
  loaded: false,
  loading: false,
  error: null,

  load: async () => {
    if (get().loading) return;
    set({ loading: true, error: null });

    if (!tauri.isTauriRuntime()) {
      set({
        loading: false,
        loaded: true,
        error: translate(useI18nStore.getState().language, "浏览器预览模式：无法读取本地知识库"),
      });
      return;
    }

    try {
      const items = await tauri.listLibraries();
      set({ items, loaded: true, loading: false });
    } catch (error) {
      set({ loading: false, loaded: true, error: describeError(error) });
    }
  },

  create: async (name) => {
    set({ error: null });
    try {
      const created = await tauri.createLibrary(name);
      const summary: LibrarySummary = {
        ...created,
        documentCount: 0,
        chunkCount: 0,
        sizeBytes: 0,
      };
      set((state) => ({ items: [summary, ...state.items] }));
      return true;
    } catch (error) {
      set({ error: describeError(error) });
      return false;
    }
  },

  rename: async (id, name) => {
    set({ error: null });
    try {
      const updated = await tauri.renameLibrary(id, name);
      set((state) => ({
        items: state.items.map((item) => (item.id === id ? { ...item, ...updated } : item)),
      }));
      return true;
    } catch (error) {
      set({ error: describeError(error) });
      return false;
    }
  },

  remove: async (id) => {
    set({ error: null });
    try {
      await tauri.deleteLibrary(id);
      set((state) => ({ items: state.items.filter((item) => item.id !== id) }));
      return true;
    } catch (error) {
      set({ error: describeError(error) });
      return false;
    }
  },

  clearError: () => set({ error: null }),
}));
