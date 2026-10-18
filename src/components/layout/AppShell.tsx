import { useEffect, useState } from "react";

import { ChatView } from "@/components/chat/ChatView";
import { Sidebar } from "@/components/layout/Sidebar";
import { StatusBar } from "@/components/layout/StatusBar";
import { LibraryView } from "@/components/library/LibraryView";
import { SettingsView } from "@/components/settings/SettingsView";
import { useAppInfoStore } from "@/stores/appInfo";
import type { AppView } from "@/types";

/** 应用外壳：左侧导航 + 内容区 + 底部状态栏。 */
export function AppShell() {
  const [view, setView] = useState<AppView>("library");
  const loadAppInfo = useAppInfoStore((state) => state.load);

  useEffect(() => {
    void loadAppInfo();
  }, [loadAppInfo]);

  return (
    <div className="bg-canvas text-ink flex h-full overflow-hidden">
      <Sidebar activeView={view} onSelect={setView} />
      <main className="flex min-w-0 flex-1 flex-col">
        <div className="lumos-aurora flex-1 overflow-y-auto">
          {view === "library" ? <LibraryView /> : null}
          {view === "chat" ? <ChatView /> : null}
          {view === "settings" ? <SettingsView /> : null}
        </div>
        <StatusBar />
      </main>
    </div>
  );
}
