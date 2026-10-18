/**
 * 在线 Demo 的事件系统替身：真实桌面版通过 Tauri 事件推送进度，
 * 浏览器 Demo 没有后台任务，`listen` 返回一个空的取消函数即可。
 */
export async function listen<T>(
  _event: string,
  _handler: (event: { payload: T }) => void,
): Promise<() => void> {
  return () => undefined;
}

export async function emit(_event: string, _payload?: unknown): Promise<void> {
  // no-op
}
