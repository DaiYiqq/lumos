/** 全局共享的领域类型。 */

/** Rust 侧 `app_info` 命令返回的应用元信息。 */
export interface AppInfo {
  /** 产品名，例如 `Lumos`。 */
  name: string;
  /** 语义化版本号，例如 `0.1.0`。 */
  version: string;
  /** 应用数据目录的绝对路径（数据库、日志、设置都存放于此）。 */
  dataDir: string;
  /** 数据目录的展示形式：家目录前缀替换为 `~`，避免暴露用户名。 */
  dataDirDisplay: string;
  /** 运行平台标识，例如 `windows` / `macos` / `linux`。 */
  platform: string;
}

/** 侧边栏一级导航。 */
export type AppView = "library" | "chat" | "settings";

/** 深浅色主题模式。 */
export type ThemeMode = "system" | "light" | "dark";
