import { create } from "zustand";

import { translate, useI18nStore } from "@/i18n";
import { getAppInfo, isTauriRuntime } from "@/lib/tauri";
import type { AppInfo } from "@/types";

interface AppInfoState {
  /** 应用元信息；浏览器预览或加载失败时为 null。 */
  info: AppInfo | null;
  /** 是否正在加载。 */
  loading: boolean;
  /** 人类可读的错误信息。 */
  error: string | null;
  /** 拉取一次应用元信息（幂等）。 */
  load: () => Promise<void>;
}

/** 应用元信息：版本号、数据目录、平台。 */
export const useAppInfoStore = create<AppInfoState>((set, get) => ({
  info: null,
  loading: false,
  error: null,
  load: async () => {
    if (get().loading || get().info !== null) return;
    set({ loading: true, error: null });

    if (!isTauriRuntime()) {
      set({
        loading: false,
        error: translate(useI18nStore.getState().language, "浏览器预览模式：未连接 Tauri 运行时"),
      });
      return;
    }

    try {
      const info = await getAppInfo();
      set({ info, loading: false });
    } catch (error) {
      set({ loading: false, error: describeInvokeError(error) });
    }
  },
}));

/** 把 Rust 侧返回的错误（通常是字符串）转成一句人话。 */
function describeInvokeError(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return "读取应用信息失败";
}
