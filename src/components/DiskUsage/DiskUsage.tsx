import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { formatSize } from "../../lib/format";

interface DiskNode {
  path: string;
  name: string;
  size: number;
  is_directory: boolean;
  children: DiskNode[];
}

const PALETTE = [
  "#5b9dff",
  "#f5a623",
  "#4ade80",
  "#b388ff",
  "#ff6b6b",
  "#43d8c9",
  "#ffd166",
];

export function DiskUsage() {
  const [path, setPath] = useState("");
  const [tree, setTree] = useState<DiskNode | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function pick() {
    const picked = await open({ directory: true });
    if (typeof picked === "string") setPath(picked);
  }

  async function run() {
    if (!path) return;
    setBusy(true);
    setError(null);
    try {
      const r = await invoke<DiskNode>("compute_disk_usage", {
        path,
        maxDepth: 4,
      });
      setTree(r);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div>
      <div className="path-bar">
        <strong>Disk Usage</strong>
        <span style={{ flex: 1 }} />
        <input
          placeholder="フォルダを指定"
          value={path}
          onChange={(e) => setPath(e.target.value)}
          style={{ width: 360 }}
        />
        <button onClick={pick}>Pick…</button>
        <button className="primary" onClick={run} disabled={busy || !path}>
          {busy ? "Scanning…" : "Scan"}
        </button>
      </div>
      <p style={{ fontSize: 12, color: "var(--fg-dim)" }}>
        FS を直接歩くので大きいフォルダだと時間かかる。max depth = 4 で固定。
      </p>

      {error && (
        <div className="empty" style={{ color: "var(--danger)" }}>
          {error}
        </div>
      )}

      {tree && (
        <>
          <p style={{ fontSize: 13 }}>
            <strong>{tree.name}</strong>: {formatSize(tree.size)}
          </p>
          <div
            style={{
              border: "1px solid var(--border)",
              borderRadius: 8,
              overflow: "hidden",
              height: 520,
              background: "var(--bg-2)",
            }}
          >
            <Treemap node={tree} horizontal depth={0} />
          </div>
        </>
      )}
    </div>
  );
}

/**
 * Slice-and-dice treemap. Alternates split direction by depth.
 * Each rectangle's flex is proportional to size.
 */
function Treemap({
  node,
  horizontal,
  depth,
}: {
  node: DiskNode;
  horizontal: boolean;
  depth: number;
}) {
  const visible = node.children.filter((c) => c.size > 0);
  if (visible.length === 0) {
    return <Tile node={node} depth={depth} />;
  }
  const total = visible.reduce((s, c) => s + c.size, 0) || 1;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: horizontal ? "row" : "column",
        width: "100%",
        height: "100%",
      }}
    >
      {visible.map((c) => (
        <div
          key={c.path}
          style={{
            flex: c.size / total,
            minWidth: 0,
            minHeight: 0,
            border: "1px solid rgba(0,0,0,0.25)",
          }}
          title={`${c.path}\n${formatSize(c.size)}`}
        >
          {c.children.length > 0 ? (
            <Treemap node={c} horizontal={!horizontal} depth={depth + 1} />
          ) : (
            <Tile node={c} depth={depth} />
          )}
        </div>
      ))}
    </div>
  );
}

function Tile({ node, depth }: { node: DiskNode; depth: number }) {
  const color = PALETTE[depth % PALETTE.length];
  return (
    <div
      style={{
        background: color,
        opacity: 0.55 + (depth % 3) * 0.1,
        height: "100%",
        width: "100%",
        padding: 4,
        color: "#000",
        fontSize: 10,
        overflow: "hidden",
        textOverflow: "ellipsis",
        whiteSpace: "nowrap",
        cursor: "default",
      }}
    >
      <div style={{ fontWeight: 600 }}>{node.name}</div>
      <div>{formatSize(node.size)}</div>
    </div>
  );
}
