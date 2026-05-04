import { useCallback, useEffect, useMemo, useState } from "react";
import clsx from "clsx";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useAppStore } from "../../lib/store";
import {
  deleteFile,
  indexDirectory,
  listDir,
  listFilesByTag,
  listSmartFolders,
  searchFiles,
} from "../../lib/tauri";
import { PRESETS } from "../../lib/presets";
import type { FileEntry } from "../../lib/types";
import { iconFor, formatSize } from "../../lib/format";
import { useKeyboard } from "../../hooks/useKeyboard";

type Mode = "grid" | "list";

const IMAGE_EXTS = ["jpg", "jpeg", "png", "gif", "webp", "bmp"];

export function FileBrowser() {
  const view = useAppStore((s) => s.view);
  const select = useAppStore((s) => s.select);
  const selected = useAppStore((s) => s.selected);
  const selectedPaths = useAppStore((s) => s.selectedPaths);
  const toggleSelected = useAppStore((s) => s.toggleSelected);
  const setSelection = useAppStore((s) => s.setSelection);
  const clearSelection = useAppStore((s) => s.clearSelection);

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
        } else if (view.kind === "preset") {
          const preset = PRESETS[view.preset];
          const r = await searchFiles(preset.query);
          if (!cancelled) {
            setEntries(r.items);
            setPath(`${preset.label} (${r.total})`);
          }
        } else if (view.kind === "smart") {
          const smarts = await listSmartFolders();
          const sf = smarts.find((s) => s.id === view.smartFolderId);
          if (sf) {
            const q = JSON.parse(sf.query_json);
            const r = await searchFiles(q);
            if (!cancelled) {
              setEntries(r.items);
              setPath(`${sf.name} (${r.total})`);
            }
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

  const handleClick = useCallback(
    (f: FileEntry, e: React.MouseEvent) => {
      const meta = e.metaKey || e.ctrlKey;
      const shift = e.shiftKey;
      if (meta) {
        toggleSelected(f.path);
        select(f);
      } else if (shift && selected) {
        // range select between selected and f
        const idxA = entries.findIndex((x) => x.path === selected.path);
        const idxB = entries.findIndex((x) => x.path === f.path);
        if (idxA >= 0 && idxB >= 0) {
          const [lo, hi] = idxA < idxB ? [idxA, idxB] : [idxB, idxA];
          setSelection(entries.slice(lo, hi + 1).map((x) => x.path));
          select(f);
        }
      } else {
        select(f);
      }
    },
    [entries, selected, select, toggleSelected, setSelection],
  );

  const enterFolder = useCallback(
    (f: FileEntry) => {
      if (f.is_directory) {
        useAppStore.getState().setView({ kind: "browse", path: f.path });
      }
    },
    [],
  );

  // Keyboard navigation
  useKeyboard(
    (e) => {
      if (entries.length === 0) return;
      const idx = selected
        ? entries.findIndex((x) => x.path === selected.path)
        : -1;

      if (e.key === "ArrowDown" || e.key === "j") {
        e.preventDefault();
        const next = entries[Math.min(idx + 1, entries.length - 1)] ?? entries[0];
        select(next);
      } else if (e.key === "ArrowUp" || e.key === "k") {
        e.preventDefault();
        const next = entries[Math.max(idx - 1, 0)] ?? entries[0];
        select(next);
      } else if (e.key === "Enter" && selected) {
        e.preventDefault();
        enterFolder(selected);
      } else if (e.key === "Escape") {
        clearSelection();
      }
    },
    [entries, selected],
  );

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

  async function bulkDelete() {
    const paths = Array.from(selectedPaths);
    if (paths.length === 0) return;
    if (!confirm(`Delete ${paths.length} item(s)? This cannot be undone yet.`))
      return;
    for (const p of paths) {
      try {
        await deleteFile(p);
      } catch (e) {
        console.error("delete failed", p, e);
      }
    }
    setEntries((prev) => prev.filter((f) => !selectedPaths.has(f.path)));
    clearSelection();
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
        {selectedPaths.size > 1 && (
          <>
            <span style={{ fontSize: 11, color: "var(--fg-dim)" }}>
              {selectedPaths.size} selected
            </span>
            <button className="danger" onClick={bulkDelete}>
              Delete
            </button>
          </>
        )}
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
              selected={selectedPaths.has(f.path)}
              onClick={(e) => handleClick(f, e)}
              onDoubleClick={() => enterFolder(f)}
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
                className={clsx({ selected: selectedPaths.has(f.path) })}
                onClick={(e) => handleClick(f, e)}
                onDoubleClick={() => enterFolder(f)}
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
  onClick: (e: React.MouseEvent) => void;
  onDoubleClick: () => void;
}) {
  // Inline image thumb when small enough; otherwise emoji icon.
  const isImage =
    !entry.is_directory && IMAGE_EXTS.includes(entry.extension ?? "");
  const showThumb = isImage && entry.size < 8 * 1024 * 1024;
  const src = showThumb ? convertFileSrc(entry.path) : null;

  return (
    <div
      className={clsx("file-card", { selected })}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      title={entry.path}
    >
      <div className="thumb">
        {src ? (
          <img
            src={src}
            alt={entry.name}
            loading="lazy"
            style={{
              width: 64,
              height: 64,
              objectFit: "cover",
              borderRadius: 4,
            }}
            onError={(e) => {
              (e.target as HTMLImageElement).style.display = "none";
            }}
          />
        ) : (
          iconFor(entry)
        )}
      </div>
      <div className="name">{entry.name}</div>
      <div className="meta">
        {entry.is_directory ? "folder" : formatSize(entry.size)}
      </div>
    </div>
  );
}
