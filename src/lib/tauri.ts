import { invoke } from "@tauri-apps/api/core";

import type { AppInfo, Library, LibrarySummary } from "@/types";

/**
 * 判断当前是否运行在 Tauri 容器内。
 *
 * 单元测试与浏览器预览（`pnpm dev`）里没有 Tauri 注入的全局对象，
 * 业务代码据此决定是否调用 Rust 命令，避免在浏览器里抛错。
 */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** 读取应用元信息（版本、数据目录、平台）。 */
export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}

/** 列出全部知识库及统计信息。 */
export async function listLibraries(): Promise<LibrarySummary[]> {
  return invoke<LibrarySummary[]>("library_list");
}

/** 创建知识库。 */
export async function createLibrary(name: string): Promise<Library> {
  return invoke<Library>("library_create", { name });
}

/** 重命名知识库。 */
export async function renameLibrary(id: string, name: string): Promise<Library> {
  return invoke<Library>("library_rename", { id, name });
}

/** 删除知识库（级联删除其文档与分块）。 */
export async function deleteLibrary(id: string): Promise<void> {
  await invoke<void>("library_delete", { id });
}
