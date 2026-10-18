import { create } from "zustand";

import type { ThemeMode } from "@/types";

const STORAGE_KEY = "lumos.theme";
const DARK_MEDIA_QUERY = "(prefers-color-scheme: dark)";

/** 根据模式与系统偏好计算当前是否应使用深色。 */
function resolveIsDark(mode: ThemeMode): boolean {
  if (mode === "dark") return true;
  if (mode === "light") return false;
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") return false;
  return window.matchMedia(DARK_MEDIA_QUERY).matches;
}

/** 把主题应用到 `<html>` 上。 */
export function applyTheme(mode: ThemeMode): void {
  const isDark = resolveIsDark(mode);
  document.documentElement.classList.toggle("dark", isDark);
  document.documentElement.style.colorScheme = isDark ? "dark" : "light";
}

interface ThemeState {
  mode: ThemeMode;
  setMode: (mode: ThemeMode) => void;
  /** 在 system → light → dark 之间循环，供快捷键与状态栏使用。 */
  cycleMode: () => void;
}

/** 主题状态：持久化到 localStorage，默认跟随系统。 */
export const useThemeStore = create<ThemeState>((set, get) => ({
  mode: readStoredMode(),
  setMode: (mode) => {
    writeStoredMode(mode);
    applyTheme(mode);
    set({ mode });
  },
  cycleMode: () => {
    const order: ThemeMode[] = ["system", "light", "dark"];
    const currentIndex = order.indexOf(get().mode);
    const next = order[(currentIndex + 1) % order.length] ?? "system";
    get().setMode(next);
  },
}));

function readStoredMode(): ThemeMode {
  if (typeof window === "undefined") return "system";
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    return stored === "light" || stored === "dark" || stored === "system" ? stored : "system";
  } catch {
    return "system";
  }
}

function writeStoredMode(mode: ThemeMode): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    // localStorage 不可用时只影响持久化，不影响本次会话
  }
}

/**
 * 启动时初始化主题，并监听系统主题变化。
 *
 * @returns 清理函数，供测试或热重载时移除监听
 */
export function initTheme(): () => void {
  const mode = useThemeStore.getState().mode;
  applyTheme(mode);

  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return () => undefined;
  }

  const media = window.matchMedia(DARK_MEDIA_QUERY);
  const handleChange = (): void => applyTheme(useThemeStore.getState().mode);
  media.addEventListener("change", handleChange);
  return () => media.removeEventListener("change", handleChange);
}
