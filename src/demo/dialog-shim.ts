/**
 * 在线 Demo 的文件选择替身：返回固定的示例路径，
 * 让“导入文件/文件夹”交互可以走通，但不会真正读取磁盘。
 */
interface DemoOpenOptions {
  directory?: boolean;
  multiple?: boolean;
}

export async function open(options?: DemoOpenOptions): Promise<string | string[] | null> {
  if (options?.directory) return "/demo-documents";
  const files = ["demo/新导入的示例文档.md", "demo/合同补充条款.md"];
  return options?.multiple ? files : (files[0] ?? null);
}
