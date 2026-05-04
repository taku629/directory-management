import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
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

interface SimGroup {
  representative: string;
  files: DupFile[];
}

type Mode = "exact" | "similar";

export function DuplicateFinder() {
  const [mode, setMode] = useState<Mode>("exact");
  const [scope, setScope] = useState<string>("");
  const [exact, setExact] = useState<DupGroup[] | null>(null);
  const [similar, setSimilar] = useState<SimGroup[] | null>(null);
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
      if (mode === "exact") {
        const r = await invoke<DupGroup[]>("find_duplicates", {
          path: scope || null,
          minSize: 4096,
        });
        setExact(r);
      } else {
        const r = await invoke<SimGroup[]>("find_similar_images", {
          path: scope || null,
          maxDistance: 5,
        });
        setSimilar(r);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div>
      <div className="path-bar">
        <strong>Duplicates</strong>
        <span style={{ flex: 1 }} />
        <select value={mode} onChange={(e) => setMode(e.target.value as Mode)}>
          <option value="exact">完全一致 (SHA-256)</option>
          <option value="similar">類似画像 (perceptual hash)</option>
        </select>
        <input
          placeholder="(全インデックス対象)"
          value={scope}
          onChange={(e) => setScope(e.target.value)}
          style={{ width: 280 }}
        />
        <button onClick={pickScope}>Pick…</button>
        <button className="primary" onClick={run} disabled={busy}>
          {busy ? "Hashing…" : "Find"}
        </button>
      </div>

      <p style={{ color: "var(--fg-dim)", fontSize: 12 }}>
        {mode === "exact" ? (
          <>SHA-256 で完全一致を検出。同サイズだけハッシュするので速い。</>
        ) : (
          <>dHash + Hamming 距離 5 以下でクラスタリング。リサイズ・JPEG 再圧縮ぐらいなら同じグループに入る。</>
        )}
      </p>

      {error && (
        <div className="empty" style={{ color: "var(--danger)" }}>
          {error}
        </div>
      )}

      {mode === "exact" && exact && (
        <ExactView groups={exact} onChanged={run} />
      )}
      {mode === "similar" && similar && (
        <SimilarView groups={similar} onChanged={run} />
      )}
    </div>
  );
}

function ExactView({
  groups,
  onChanged,
}: {
  groups: DupGroup[];
  onChanged: () => void;
}) {
  const wasted = groups.reduce(
    (acc, g) => acc + g.size * (g.files.length - 1),
    0,
  );

  if (groups.length === 0)
    return <div className="empty">重複なし。健全。</div>;
  return (
    <div>
      <p style={{ fontSize: 13 }}>
        <strong>{groups.length}</strong> グループ · 約{" "}
        <strong>{formatSize(wasted)}</strong> 節約できる
      </p>
      {groups.map((g) => (
        <ExactGroup key={g.hash} g={g} onChanged={onChanged} />
      ))}
    </div>
  );
}

function ExactGroup({
  g,
  onChanged,
}: {
  g: DupGroup;
  onChanged: () => void;
}) {
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
          style={{ display: "flex", padding: "4px 0", fontSize: 12 }}
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

function SimilarView({
  groups,
  onChanged,
}: {
  groups: SimGroup[];
  onChanged: () => void;
}) {
  if (groups.length === 0)
    return <div className="empty">類似画像見つからず。</div>;
  return (
    <div>
      <p style={{ fontSize: 13 }}>
        <strong>{groups.length}</strong> クラスタ
      </p>
      {groups.map((g, i) => (
        <SimilarGroupCard key={i} g={g} onChanged={onChanged} />
      ))}
    </div>
  );
}

function SimilarGroupCard({
  g,
  onChanged,
}: {
  g: SimGroup;
  onChanged: () => void;
}) {
  const [keep, setKeep] = useState<string>(g.files[0].path);
  async function deleteOthers() {
    const others = g.files.filter((f) => f.path !== keep);
    if (!confirm(`${others.length} 個削除。OK?`)) return;
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
        <span style={{ fontSize: 12 }}>{g.files.length} 枚</span>
        <span style={{ flex: 1 }} />
        <button className="danger" onClick={deleteOthers}>
          Keep selected, delete others
        </button>
      </div>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(140px, 1fr))",
          gap: 8,
        }}
      >
        {g.files.map((f) => (
          <label
            key={f.path}
            style={{
              display: "block",
              border:
                keep === f.path
                  ? "2px solid var(--accent)"
                  : "2px solid transparent",
              borderRadius: 6,
              padding: 4,
              cursor: "pointer",
              background: "var(--bg-3)",
            }}
            title={f.path}
          >
            <img
              src={convertFileSrc(f.path)}
              alt={f.name}
              loading="lazy"
              style={{
                width: "100%",
                height: 100,
                objectFit: "cover",
                borderRadius: 4,
                background: "#000",
              }}
              onError={(e) =>
                ((e.target as HTMLImageElement).style.display = "none")
              }
            />
            <input
              type="radio"
              name={`keep-${g.representative}`}
              checked={keep === f.path}
              onChange={() => setKeep(f.path)}
              style={{ marginRight: 4 }}
            />
            <span style={{ fontSize: 10 }}>{f.name}</span>
          </label>
        ))}
      </div>
    </div>
  );
}
