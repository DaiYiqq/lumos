/** 字符串工具集合。 */

/**
 * 合并 className，过滤掉假值，避免引入额外依赖。
 *
 * @param classes 任意 className 片段
 * @returns 以空格连接的 className 字符串
 */
export function cn(...classes: Array<string | false | null | undefined>): string {
  return classes.filter(Boolean).join(" ");
}

/**
 * 把字节数格式化为人类可读的大小。
 *
 * @param bytes 字节数
 * @returns 例如 `12.4 MB`
 */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "—";
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(1)} ${units[unitIndex]}`;
}
