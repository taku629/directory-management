import { useEffect, useMemo, useState } from "react";
import clsx from "clsx";
import { useAppStore } from "../../lib/store";
import {
  indexDirectory,
  listDir,
  listFilesByTag,
  searchFiles,
} from "../../lib/tauri";
import type { FileEntry } from "../../lib/types";
import { iconFor, formatSize } from "../../lib/format";

type Mode = "grid" | "list";

export function FileBrowser() {
  const view = useAppStore((s) => s.view);
  const select = useAppStore((s) => s.select);
  const selected = useAppStore((s) => s.selected);
  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [path, setPath] = useState<string>("");
  const [mode, setMode] = useState<Mode>("grid");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    async function load() {
      try {
        if (view.kind === "browse") {
          if (!view.path) {
            setEntries([]);
            setPath("");
            return;
          }
          const r = await listDir(view.path);
          if (!cancelled) {
            setEntries(r.entries);
            setPath(r.path);
          }
        } else if (view.kind === "tag") {
          const items = await listFilesByTag(view.tagId);
          if (!cancelled) {
            setEntries(items);
            setPath(`tag:${view.tagId}`);
          }
        } else if (view.kind === "search") {
          const r = await searchFiles({ text: view.text || undefined });
          if (!cancelled) {
            setEntries(r.items);
            setPath(`search:"${view.text}" (${r.total} results)`);
          }
        }
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    }
    load();
    return () => {
      cancelled = true;
    };
  }, [view]);

  const breadcrumbs = useMemo(() => {
    if (view.kind !== "browse" || !path) return null;
    const parts = path.split(/[\\/]/).filter(Boolean);
    return parts.join(" / ");
  }, [view, path]);

  async function reindex() {
    if (view.kind !== "browse" || !path) return;
    setBusy(true);
    try {
      const r = await indexDirectory(path);
      alert(
        `Indexed ${r.indexed} files (skipped ${r.skipped}, errors ${r.errors})`,
      );
    } catch (e) {
      alert(`Index failed: ${e}`);
    } finally {
      setBusy(false);
    }
  }

  if (!path && view.kind === "browse") {
    return (
      <div className="empty">
        Click <strong>Open Folder…</strong> in the top bar, or pick a favorite
        from the sidebar.
      </div>
    );
  }

  return (
    <div>
      <div className="path-bar">
        <span>{breadcrumbs ?? path}</span>
        <span style={{ flex: 1 }} />
        <button onClick={() => setMode(mode === "grid" ? "list" : "grid")}>
          {mode === "grid" ? "List" : "Grid"}
        </button>
        {view.kind === "browse" && (
          <button onClick={reindex} disabled={busy}>
            {busy ? "Indexing…" : "Index this folder"}
          </button>
        )}
      </div>

      {error && (
        <div className="empty" style={{ color: "var(--danger)" }}>
          {error}
        </div>
      )}

      {entries.length === 0 && !error && (
        <div className="empty">No files here.</div>
      )}

      {mode === "grid" ? (
        <div className="file-grid">
          {entries.map((f) => (
            <Card
              key={f.path}
              entry={f}
              selected={selected?.path === f.path}
              onClick={() => select(f)}
              onDoubleClick={() => {
                if (f.is_directory)
                  useAppStore
                    .getState()
                    .setView({ kind: "browse", path: f.path });
              }}
            />
          ))}
        </div>
      ) : (
        <table className="file-list">
          <thead>
            <tr>
              <th>Name</th>
              <th>Size</th>
              <th>Modified</th>
              <th>Type</th>
            </tr>
          </thead>
          <tbody>
            {entries.map((f) => (
              <tr
                key={f.path}
                className={clsx({ selected: selected?.path === f.path })}
                onClick={() => select(f)}
                onDoubleClick={() => {
                  if (f.is_directory)
                    useAppStore
                      .getState()
                      .setView({ kind: "browse", path: f.path });
                }}
              >
                <td>
                  <span style={{ marginRight: 6 }}>{iconFor(f)}</span>
                  {f.name}
                </td>
                <td>{f.is_directory ? "—" : formatSize(f.size)}</td>
                <td>{new Date(f.modified_at * 1000).toLocaleString()}</td>
                <td>{f.extension ?? (f.is_directory ? "folder" : "")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function Card({
  entry,
  selected,
  onClick,
  onDoubleClick,
}: {
  entry: FileEntry;
  selected: boolean;
  onClick: () => void;
  onDoubleClick: () => void;
}) {
  return (
    <div
      className={clsx("file-card", { selected })}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      title={entry.path}
    >
      <div className="thumb">{iconFor(entry)}</div>
      <div className="name">{entry.name}</div>
      <div className="meta">
        {entry.is_directory ? "folder" : formatSize(entry.size)}
      </div>
    </div>
  );
}
