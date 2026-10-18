import { useAppInfoStore } from "@/stores/appInfo";

/** 窗口底部状态栏：本地模式、数据目录、版本号。 */
export function StatusBar() {
  const info = useAppInfoStore((state) => state.info);
  const error = useAppInfoStore((state) => state.error);

  const dataDir = info?.dataDirDisplay ?? error ?? "正在读取数据目录…";
  const version = info ? `${info.name} v${info.version} · ${info.platform}` : "Lumos";

  return (
    <footer className="border-line bg-surface text-ink-faint flex h-8 shrink-0 items-center gap-3 border-t px-4 text-[11px]">
      <span className="text-ink-muted inline-flex items-center gap-1.5 font-medium">
        <span className="size-1.5 rounded-full bg-emerald-500" aria-hidden="true" />
        本地模式
      </span>
      <span className="text-line" aria-hidden="true">
        |
      </span>
      <span className="min-w-0 flex-1 truncate" title={info?.dataDir ?? dataDir}>
        {dataDir}
      </span>
      <span className="shrink-0">{version}</span>
    </footer>
  );
}
