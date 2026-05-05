import { useEffect, useState } from "react";
import { Sidebar } from "./components/Sidebar/Sidebar";
import { FileBrowser } from "./components/FileBrowser/FileBrowser";
import { Preview } from "./components/Preview/Preview";
import { SearchBar } from "./components/SearchBar/SearchBar";
import { TagManager } from "./components/TagManager/TagManager";
import { RuleEditor } from "./components/RuleEditor/RuleEditor";
import { DuplicateFinder } from "./components/DuplicateFinder/DuplicateFinder";
import { DiskUsage } from "./components/DiskUsage/DiskUsage";
import { AIPanel } from "./components/AIPanel/AIPanel";
import { Settings } from "./components/Settings/Settings";
import { Onboarding } from "./components/Onboarding/Onboarding";
import { Pricing } from "./components/Pricing/Pricing";
import { TrialBadge } from "./components/TrialBadge";
import { useAppStore } from "./lib/store";
import { getSetting, listTags, listWatchedRoots, setSetting } from "./lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";
import { initLocale, useT } from "./lib/i18n";

export default function App() {
  const view = useAppStore((s) => s.view);
  const setView = useAppStore((s) => s.setView);
  const setTags = useAppStore((s) => s.setTags);
  const [status, setStatus] = useState("ready");
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [ready, setReady] = useState(false);
  const t = useT();

  useEffect(() => {
    (async () => {
      await initLocale();
      const seen = await getSetting("onboarded");
      const roots = await listWatchedRoots();
      if (!seen && roots.length === 0) setShowOnboarding(true);
      setReady(true);
    })();
    listTags().then(setTags).catch((e) => setStatus(`tags: ${e}`));
  }, [setTags]);

  async function chooseFolder() {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") {
      setView({ kind: "browse", path: picked });
    }
  }

  if (!ready) return null;

  return (
    <div className="app">
      <header className="topbar">
        <span className="brand">{t("app.brand")}</span>
        <button onClick={chooseFolder}>{t("topbar.openFolder")}</button>
        <SearchBar onStatus={setStatus} />
        <span className="spacer" />
        <TrialBadge onUpgrade={() => setView({ kind: "pricing" })} />
        <button onClick={() => setView({ kind: "settings" })}>⚙</button>
      </header>

      {showOnboarding && (
        <Onboarding
          onClose={async () => {
            await setSetting("onboarded", "1");
            setShowOnboarding(false);
          }}
        />
      )}

      <Sidebar />

      <main className="main">
        {view.kind === "browse" && <FileBrowser />}
        {view.kind === "tag" && <TagManager.View tagId={view.tagId} />}
        {view.kind === "search" && <FileBrowser />}
        {view.kind === "preset" && <FileBrowser />}
        {view.kind === "smart" && <FileBrowser />}
        {view.kind === "duplicates" && <DuplicateFinder />}
        {view.kind === "disk-usage" && <DiskUsage />}
        {view.kind === "rules" && <RuleEditor />}
        {view.kind === "ai" && <AIPanel />}
        {view.kind === "settings" && <Settings />}
        {view.kind === "pricing" && <Pricing />}
      </main>

      <aside className="inspector">
        <Preview />
      </aside>

      <footer className="statusbar">
        <span>{status}</span>
        <span className="spacer" style={{ flex: 1 }} />
        <span>Sift v0.1.0</span>
      </footer>
    </div>
  );
}
