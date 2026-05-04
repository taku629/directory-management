import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { deleteFile, openInExplorer } from "../../lib/tauri";
import { formatSize } from "../../lib/format";

interface DupFile {
  id: number;
  path: string;
  name: string;
  modified_at: number;
}

interface DupGroup {
  hash: string;
  size: number;
  files: DupFile[];
}

export function DuplicateFinder() {
  const [scope, setScope] = useState<string>("");
  const [groups, setGroups] = useState<DupGroup[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function pickScope() {
    const picked = await open({ directory: true });
    if (typeof picked === "string") setScope(picked);
  }

  async function run() {
    setBusy(true);
    setError(null);
    try {
      const r = await invoke<DupGroup[]>("find_duplicates", {
        path: scope || null,
        minSize: 4096,
      });
      setGroups(r);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  const wasted = (groups ?? []).reduce(
    (acc, g) => acc + g.size * (g.files.length - 1),
    0,
  );

  return (
    <div>
      <div className="path-bar">
        <strong>Duplicates</strong>
        <span style={{ flex: 1 }} />
        <input
          placeholder="(全インデックス対象) もしくはパスを指定"
          value={scope}
          onChange={(e) => setScope(e.target.value)}
          style={{ width: 320 }}
        />
        <button onClick={pickScope}>Pick…</button>
        <button className="primary" onClick={run} disabled={busy}>
          {busy ? "Hashing…" : "Find"}
        </button>
      </div>

      <p style={{ color: "var(--fg-dim)", fontSize: 12 }}>
        SHA-256 で完全一致を検出。同じサイズのファイルだけハッシュするので
        初回でもそこそこ速い。インデックスしたフォルダが対象。
      </p>

      {error && (
        <div className="empty" style={{ color: "var(--danger)" }}>
          {error}
        </div>
      )}

      {groups && groups.length === 0 && (
        <div className="empty">重複なし。健全。</div>
      )}

      {groups && groups.length > 0 && (
        <div>
          <p style={{ fontSize: 13 }}>
            <strong>{groups.length}</strong> グループ ·
            約 <strong>{formatSize(wasted)}</strong> 節約できる
          </p>
          {groups.map((g) => (
            <Group key={g.hash} g={g} onChanged={run} />
          ))}
        </div>
      )}
    </div>
  );
}

function Group({ g, onChanged }: { g: DupGroup; onChanged: () => void }) {
  // sort by modified_at asc, default keep oldest
  const [keep, setKeep] = useState<string>(g.files[0].path);

  async function deleteOthers() {
    const others = g.files.filter((f) => f.path !== keep);
    if (
      !confirm(
        `${others.length} 個削除。残すのは ${keep.split(/[\\/]/).pop()}。OK?`,
      )
    )
      return;
    for (const f of others) {
      try {
        await deleteFile(f.path);
      } catch (e) {
        console.error(e);
      }
    }
    onChanged();
  }

  return (
    <div
      style={{
        background: "var(--bg-2)",
        border: "1px solid var(--border)",
        borderRadius: 8,
        padding: 12,
        marginBottom: 12,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", marginBottom: 8 }}>
        <code style={{ fontSize: 11, color: "var(--fg-dim)" }}>
          {g.hash.slice(0, 12)}…
        </code>
        <span style={{ marginLeft: 8, fontSize: 12 }}>
          {formatSize(g.size)} × {g.files.length}
        </span>
        <span style={{ flex: 1 }} />
        <button className="danger" onClick={deleteOthers}>
          Keep selected, delete others
        </button>
      </div>
      {g.files.map((f) => (
        <label
          key={f.path}
          style={{
            display: "flex",
            alignItems: "center",
            padding: "4px 0",
            fontSize: 12,
          }}
        >
          <input
            type="radio"
            name={`keep-${g.hash}`}
            checked={keep === f.path}
            onChange={() => setKeep(f.path)}
          />
          <span
            style={{ flex: 1, marginLeft: 8, wordBreak: "break-all" }}
            title={f.path}
          >
            {f.path}
          </span>
          <span style={{ color: "var(--fg-dim)", fontSize: 10 }}>
            {new Date(f.modified_at * 1000).toLocaleDateString()}
          </span>
          <button
            style={{ marginLeft: 8 }}
            onClick={() => openInExplorer(f.path)}
          >
            Reveal
          </button>
        </label>
      ))}
    </div>
  );
}
