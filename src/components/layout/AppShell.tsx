import { useEffect } from "react";

import { ChatView } from "@/components/chat/ChatView";
import { DemoBanner } from "@/components/layout/DemoBanner";
import { Sidebar } from "@/components/layout/Sidebar";
import { StatusBar } from "@/components/layout/StatusBar";
import { LibraryView } from "@/components/library/LibraryView";
import { SearchView } from "@/components/search/SearchView";
import { SettingsView } from "@/components/settings/SettingsView";
import { useAppInfoStore } from "@/stores/appInfo";
import { useUiStore } from "@/stores/ui";

/** 应用外壳：左侧导航 + 内容区 + 底部状态栏。 */
export function AppShell() {
  const activeView = useUiStore((state) => state.activeView);
  const setActiveView = useUiStore((state) => state.setActiveView);
  const loadAppInfo = useAppInfoStore((state) => state.load);
  const demo = import.meta.env.MODE === "demo";

  useEffect(() => {
    void loadAppInfo();
  }, [loadAppInfo]);

  return (
    <div className="bg-canvas text-ink flex h-full flex-col overflow-hidden">
      {demo ? <DemoBanner /> : null}
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <Sidebar activeView={activeView} onSelect={setActiveView} />
        <main className="flex min-w-0 flex-1 flex-col">
          <div className="lumos-aurora flex min-h-0 flex-1 flex-col overflow-hidden">
            {activeView === "library" ? <LibraryView /> : null}
            {activeView === "search" ? <SearchView /> : null}
            {activeView === "chat" ? <ChatView /> : null}
            {activeView === "settings" ? <SettingsView /> : null}
          </div>
          <StatusBar />
        </main>
      </div>
    </div>
  );
}
