import { create } from "zustand";

import type { AppView } from "@/types";

interface UiState {
  /** 当前一级页面。 */
  activeView: AppView;
  /** 当前选中的知识库（知识库页与问答页共享）。 */
  selectedLibraryId: string | null;
  /** 切换一级页面。 */
  setActiveView: (view: AppView) => void;
  /** 选中知识库。 */
  setSelectedLibraryId: (id: string | null) => void;
}

/** 全局 UI 状态：页面导航与跨页面选中的知识库。 */
export const useUiStore = create<UiState>((set) => ({
  activeView: "library",
  selectedLibraryId: null,
  setActiveView: (activeView) => set({ activeView }),
  setSelectedLibraryId: (selectedLibraryId) => set({ selectedLibraryId }),
}));
