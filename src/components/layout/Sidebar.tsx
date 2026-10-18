import { Library, MessagesSquare, Moon, Settings, Sun, SunMoon } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Logo } from "@/components/ui/Logo";
import { cn } from "@/lib/utils";
import { useThemeStore } from "@/stores/theme";
import type { AppView } from "@/types";

interface NavItem {
  view: AppView;
  label: string;
  icon: LucideIcon;
}

const NAV_ITEMS: NavItem[] = [
  { view: "library", label: "知识库", icon: Library },
  { view: "chat", label: "问答", icon: MessagesSquare },
  { view: "settings", label: "设置", icon: Settings },
];

interface SidebarProps {
  /** 当前一级页面。 */
  activeView: AppView;
  /** 切换一级页面。 */
  onSelect: (view: AppView) => void;
}

/** 左侧主导航栏。 */
export function Sidebar({ activeView, onSelect }: SidebarProps) {
  const mode = useThemeStore((state) => state.mode);

  const ThemeIcon = mode === "dark" ? Moon : mode === "light" ? Sun : SunMoon;

  return (
    <aside className="border-line bg-surface flex w-64 shrink-0 flex-col border-r">
      <div className="flex items-center gap-3 px-5 pt-5 pb-6">
        <Logo size={30} />
        <div className="min-w-0">
          <div className="text-ink text-[15px] leading-none font-semibold tracking-tight">
            Lumos
          </div>
          <div className="text-ink-faint mt-1 truncate text-[11px] leading-none">
            本地优先的引用式知识库
          </div>
        </div>
      </div>

      <nav className="flex flex-col gap-1 px-3" aria-label="主导航">
        {NAV_ITEMS.map(({ view, label, icon: Icon }) => {
          const active = view === activeView;
          return (
            <button
              key={view}
              type="button"
              onClick={() => onSelect(view)}
              aria-current={active ? "page" : undefined}
              className={cn(
                "flex h-9 items-center gap-2.5 rounded-lg px-3 text-sm transition-colors",
                "focus-visible:outline-brand focus-visible:outline-2 focus-visible:outline-offset-2",
                active
                  ? "bg-brand-soft text-brand font-medium"
                  : "text-ink-muted hover:bg-surface-raised hover:text-ink",
              )}
            >
              <Icon size={16} strokeWidth={2} aria-hidden="true" />
              {label}
            </button>
          );
        })}
      </nav>

      <div className="border-line mt-auto flex flex-col gap-3 border-t px-3 py-3">
        <button
          type="button"
          onClick={() => useThemeStore.getState().cycleMode()}
          title={`主题：${themeLabel(mode)}（点击切换）`}
          className="text-ink-muted hover:bg-surface-raised hover:text-ink flex h-8 items-center gap-2 rounded-lg px-2.5 text-[13px] transition-colors"
        >
          <ThemeIcon size={15} aria-hidden="true" />
          <span>主题：{themeLabel(mode)}</span>
        </button>

        <div className="text-ink-faint px-2.5 pb-1 text-[11px] leading-relaxed">
          所有数据只保存在本机
        </div>
        <div className="sr-only" aria-live="polite">
          当前主题：{themeLabel(mode)}
        </div>
      </div>
    </aside>
  );
}

/** 主题模式的中文标签。 */
function themeLabel(mode: "system" | "light" | "dark"): string {
  if (mode === "dark") return "深色";
  if (mode === "light") return "浅色";
  return "跟随系统";
}
