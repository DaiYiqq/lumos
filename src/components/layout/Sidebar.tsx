import { Library, MessagesSquare, Moon, Search, Settings, Sun, SunMoon } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Logo } from "@/components/ui/Logo";
import { useT } from "@/i18n";
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
  { view: "search", label: "搜索", icon: Search },
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
  const t = useT();

  const ThemeIcon = mode === "dark" ? Moon : mode === "light" ? Sun : SunMoon;
  const themeName = t(themeLabelSource(mode));

  return (
    <aside className="border-line bg-surface flex w-64 shrink-0 flex-col border-r">
      <div className="flex items-center gap-3 px-5 pt-5 pb-6">
        <Logo size={30} />
        <div className="min-w-0">
          <div className="text-ink text-[15px] leading-none font-semibold tracking-tight">
            Lumos
          </div>
          <div className="text-ink-faint mt-1 truncate text-[11px] leading-none">
            {t("本地优先的引用式知识库")}
          </div>
        </div>
      </div>

      <nav className="flex flex-col gap-1 px-3" aria-label={t("主导航")}>
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
              {t(label)}
            </button>
          );
        })}
      </nav>

      <div className="border-line mt-auto flex flex-col gap-3 border-t px-3 py-3">
        <button
          type="button"
          onClick={() => useThemeStore.getState().cycleMode()}
          title={t("主题：{mode}（点击切换）", { mode: themeName })}
          className="text-ink-muted hover:bg-surface-raised hover:text-ink flex h-8 items-center gap-2 rounded-lg px-2.5 text-[13px] transition-colors"
        >
          <ThemeIcon size={15} aria-hidden="true" />
          <span>{t("主题：{mode}", { mode: themeName })}</span>
        </button>

        <div className="text-ink-faint px-2.5 pb-1 text-[11px] leading-relaxed">
          {t("所有数据只保存在本机")}
        </div>
        <div className="sr-only" aria-live="polite">
          {t("当前主题：{mode}", { mode: themeName })}
        </div>
      </div>
    </aside>
  );
}

/** 主题模式的中文标签（作为词典键）。 */
function themeLabelSource(mode: "system" | "light" | "dark"): string {
  if (mode === "dark") return "深色";
  if (mode === "light") return "浅色";
  return "跟随系统";
}
