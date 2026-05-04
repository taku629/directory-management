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
import { useAppStore } from "./lib/store";
import { listTags } from "./lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";

export default function App() {
  const view = useAppStore((s) => s.view);
  const setView = useAppStore((s) => s.setView);
  const setTags = useAppStore((s) => s.setTags);
  const [status, setStatus] = useState("ready");

  useEffect(() => {
    listTags().then(setTags).catch((e) => setStatus(`tags: ${e}`));
  }, [setTags]);

  async function chooseFolder() {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") {
      setView({ kind: "browse", path: picked });
    }
  }

  return (
    <div className="app">
      <header className="topbar">
        <span className="brand">Sift</span>
        <button onClick={chooseFolder}>Open Folder…</button>
        <SearchBar onStatus={setStatus} />
        <span className="spacer" />
        <button onClick={() => setView({ kind: "settings" })}>⚙</button>
      </header>

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
