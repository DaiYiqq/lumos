import { create } from "zustand";

import { translate, useI18nStore } from "@/i18n";
import * as tauri from "@/lib/tauri";
import type { DocumentSummary, IngestJob } from "@/types";

interface DocumentsState {
  /** 当前知识库的文档。 */
  documents: DocumentSummary[];
  /** 当前知识库的导入任务（最新在前）。 */
  jobs: IngestJob[];
  /** 是否正在加载。 */
  loading: boolean;
  /** 队列是否暂停。 */
  paused: boolean;
  /** 错误文案。 */
  error: string | null;
  /** 加载指定知识库的文档与任务。 */
  load: (libraryId: string) => Promise<void>;
  /** 清空（切换知识库时调用）。 */
  clear: () => void;
  /** 暂停/恢复队列。 */
  setPaused: (paused: boolean) => Promise<void>;
  /** 取消任务。 */
  cancelJob: (jobId: string) => Promise<void>;
  /** 删除文档。 */
  removeDocument: (id: string) => Promise<void>;
}

/** 把未知错误转成一句可展示的话。 */
function describeError(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return "操作失败，请重试";
}

/** 文档与导入任务状态。 */
export const useDocumentsStore = create<DocumentsState>((set) => ({
  documents: [],
  jobs: [],
  loading: false,
  paused: false,
  error: null,

  load: async (libraryId) => {
    if (!tauri.isTauriRuntime()) {
      set({
        loading: false,
        error: translate(useI18nStore.getState().language, "浏览器预览模式：无法读取本地文档"),
      });
      return;
    }

    set({ loading: true });
    try {
      const [documents, jobs] = await Promise.all([
        tauri.listDocuments(libraryId),
        tauri.listIngestJobs(libraryId),
        tauri.getIngestQueueState().then((paused) => set({ paused })),
      ]);
      set({ documents, jobs, loading: false, error: null });
    } catch (error) {
      set({ loading: false, error: describeError(error) });
    }
  },

  clear: () => set({ documents: [], jobs: [], error: null, paused: false }),

  setPaused: async (paused) => {
    try {
      const next = paused ? await tauri.pauseIngestQueue() : await tauri.resumeIngestQueue();
      set({ paused: next });
    } catch (error) {
      set({ error: describeError(error) });
    }
  },

  cancelJob: async (jobId) => {
    try {
      await tauri.cancelIngestJob(jobId);
      set((state) => ({
        jobs: state.jobs.map((job) =>
          job.id === jobId ? { ...job, status: "cancelled", message: "任务已取消" } : job,
        ),
      }));
    } catch (error) {
      set({ error: describeError(error) });
    }
  },

  removeDocument: async (id) => {
    try {
      await tauri.deleteDocument(id);
      set((state) => ({ documents: state.documents.filter((document) => document.id !== id) }));
    } catch (error) {
      set({ error: describeError(error) });
    }
  },
}));
