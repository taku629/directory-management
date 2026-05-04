import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  createRule,
  deleteRule,
  listOperations,
  listRules,
  runRuleNow,
  startWatcher,
  stopWatcher,
  undoOperation,
  updateRule,
  watcherStatus,
} from "../../lib/tauri";
import type {
  OperationRow,
  PlannedOp,
  Rule,
  RuleInput,
  WatcherStatus,
} from "../../lib/types";

const SAMPLE_CONDITIONS = `[
  { "type": "ExtensionIn", "value": ["pdf"] },
  { "type": "SizeBetween", "value": { "min": 1000 } }
]`;

const SAMPLE_ACTIONS = `[
  { "type": "MoveTo", "value": "/Users/me/Documents/{year}/{month}" },
  { "type": "Notify", "value": "PDF moved" }
]`;

export function RuleEditor() {
  const [rules, setRules] = useState<Rule[]>([]);
  const [editing, setEditing] = useState<Rule | "new" | null>(null);
  const [watcher, setWatcher] = useState<WatcherStatus | null>(null);
  const [history, setHistory] = useState<OperationRow[]>([]);
  const [planResult, setPlanResult] = useState<PlannedOp[] | null>(null);
  const [busy, setBusy] = useState(false);

  async function refresh() {
    const [r, w, h] = await Promise.all([
      listRules(),
      watcherStatus(),
      listOperations(20),
    ]);
    setRules(r);
    setWatcher(w);
    setHistory(h);
  }
  useEffect(() => {
    refresh().catch(console.error);
  }, []);

  async function toggleWatcher() {
    setBusy(true);
    try {
      if (watcher?.running) await stopWatcher();
      else await startWatcher();
      setWatcher(await watcherStatus());
    } finally {
      setBusy(false);
    }
  }

  async function dryRun(rule: Rule) {
    setBusy(true);
    setPlanResult(null);
    try {
      const r = await runRuleNow(rule.id, true);
      setPlanResult(r.plan);
      alert(`${r.matched} ファイル該当 (dry run)`);
    } catch (e) {
      alert(`Error: ${e}`);
    } finally {
      setBusy(false);
    }
  }

  async function runReal(rule: Rule) {
    if (!confirm(`${rule.name} を実行する? (移動/リネームが起きる)`)) return;
    setBusy(true);
    try {
      const r = await runRuleNow(rule.id, false);
      alert(`${r.applied} 件のアクションを適用 (${r.matched} ファイル該当)`);
      await refresh();
    } catch (e) {
      alert(`Error: ${e}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div>
      <div className="path-bar">
        <strong>Rules</strong>
        <span style={{ flex: 1 }} />
        <span style={{ fontSize: 12, color: "var(--fg-dim)" }}>
          Watcher: {watcher?.running ? "▶ 動作中" : "⏸ 停止"}
        </span>
        <button onClick={toggleWatcher} disabled={busy}>
          {watcher?.running ? "Stop" : "Start"} watcher
        </button>
        <button className="primary" onClick={() => setEditing("new")}>
          + New rule
        </button>
      </div>

      <p style={{ fontSize: 12, color: "var(--fg-dim)" }}>
        Watcher を ON にしておくと、ルールの watched_path 配下にファイルが来たとき自動でアクション発火。
        OFF でも "Run now" で手動実行できる。
      </p>

      {rules.length === 0 ? (
        <div className="empty">まだルールなし。"New rule" で 1 個作る。</div>
      ) : (
        rules.map((r) => (
          <RuleRow
            key={r.id}
            rule={r}
            onEdit={() => setEditing(r)}
            onDryRun={() => dryRun(r)}
            onRun={() => runReal(r)}
            onDelete={async () => {
              if (confirm(`Delete "${r.name}"?`)) {
                await deleteRule(r.id);
                refresh();
              }
            }}
          />
        ))
      )}

      {planResult && (
        <div
          style={{
            marginTop: 16,
            padding: 12,
            border: "1px solid var(--border)",
            borderRadius: 8,
            background: "var(--bg-2)",
          }}
        >
          <h3 style={{ marginTop: 0 }}>Plan ({planResult.length} ops)</h3>
          {planResult.slice(0, 50).map((op, i) => (
            <div
              key={i}
              style={{
                fontSize: 11,
                fontFamily: "monospace",
                padding: "2px 0",
              }}
            >
              <span style={{ color: "var(--accent)" }}>{op.action}</span>{" "}
              <span style={{ color: "var(--fg-dim)" }}>{op.file_path}</span>
              {" → "}
              {op.detail}
            </div>
          ))}
          {planResult.length > 50 && (
            <div style={{ fontSize: 11, color: "var(--fg-dim)" }}>
              … and {planResult.length - 50} more
            </div>
          )}
        </div>
      )}

      <h3 style={{ marginTop: 24 }}>History (recent)</h3>
      {history.length === 0 ? (
        <div className="empty">操作履歴なし。</div>
      ) : (
        <table className="file-list">
          <thead>
            <tr>
              <th>When</th>
              <th>Type</th>
              <th>Trigger</th>
              <th>Detail</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {history.map((op) => (
              <tr key={op.id}>
                <td style={{ fontSize: 11 }}>
                  {new Date(op.created_at * 1000).toLocaleString()}
                </td>
                <td>{op.op_type}</td>
                <td style={{ fontSize: 11 }}>{op.triggered_by}</td>
                <td
                  style={{
                    fontFamily: "monospace",
                    fontSize: 10,
                    maxWidth: 360,
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {op.payload_json}
                </td>
                <td>
                  {op.undone ? (
                    <span style={{ color: "var(--fg-dim)" }}>undone</span>
                  ) : (
                    op.inverse_json && (
                      <button
                        onClick={async () => {
                          try {
                            await undoOperation(op.id);
                            await refresh();
                          } catch (e) {
                            alert(`Undo failed: ${e}`);
                          }
                        }}
                      >
                        Undo
                      </button>
                    )
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {editing && (
        <RuleForm
          existing={editing === "new" ? undefined : editing}
          onClose={() => {
            setEditing(null);
            refresh();
          }}
        />
      )}
    </div>
  );
}

function RuleRow({
  rule,
  onEdit,
  onDryRun,
  onRun,
  onDelete,
}: {
  rule: Rule;
  onEdit: () => void;
  onDryRun: () => void;
  onRun: () => void;
  onDelete: () => void;
}) {
  return (
    <div
      style={{
        background: "var(--bg-2)",
        border: "1px solid var(--border)",
        borderRadius: 8,
        padding: 12,
        marginBottom: 8,
        display: "flex",
        alignItems: "center",
      }}
    >
      <div style={{ flex: 1 }}>
        <div>
          <strong>{rule.name}</strong>
          {!rule.enabled && (
            <span
              style={{
                marginLeft: 8,
                fontSize: 10,
                color: "var(--fg-dim)",
              }}
            >
              disabled
            </span>
          )}
        </div>
        <div style={{ fontSize: 11, color: "var(--fg-dim)" }}>
          watch: {rule.watched_path}
          {" · "}cond: {rule.conditions.length}
          {" · "}act: {rule.actions.length}
          {rule.last_run_at && (
            <>
              {" · "}last:{" "}
              {new Date(rule.last_run_at * 1000).toLocaleString()}
            </>
          )}
        </div>
      </div>
      <button onClick={onDryRun}>Dry run</button>
      <button onClick={onRun} className="primary" style={{ marginLeft: 4 }}>
        Run now
      </button>
      <button onClick={onEdit} style={{ marginLeft: 4 }}>
        Edit
      </button>
      <button onClick={onDelete} className="danger" style={{ marginLeft: 4 }}>
        Delete
      </button>
    </div>
  );
}

function RuleForm({
  existing,
  onClose,
}: {
  existing?: Rule;
  onClose: () => void;
}) {
  const [name, setName] = useState(existing?.name ?? "");
  const [enabled, setEnabled] = useState(existing?.enabled ?? true);
  const [watchedPath, setWatchedPath] = useState(existing?.watched_path ?? "");
  const [priority, setPriority] = useState(existing?.priority ?? 0);
  const [conditions, setConditions] = useState(
    existing
      ? JSON.stringify(existing.conditions, null, 2)
      : SAMPLE_CONDITIONS,
  );
  const [actions, setActions] = useState(
    existing ? JSON.stringify(existing.actions, null, 2) : SAMPLE_ACTIONS,
  );

  async function pickPath() {
    const picked = await open({ directory: true });
    if (typeof picked === "string") setWatchedPath(picked);
  }

  async function save() {
    try {
      JSON.parse(conditions);
      JSON.parse(actions);
    } catch (e) {
      alert(`JSON 不正: ${e}`);
      return;
    }
    const input: RuleInput = {
      name,
      enabled,
      watched_path: watchedPath,
      conditions_json: conditions,
      actions_json: actions,
      priority,
    };
    try {
      if (existing) await updateRule(existing.id, input);
      else await createRule(input);
      onClose();
    } catch (e) {
      alert(`保存失敗: ${e}`);
    }
  }

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.5)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 1000,
      }}
      onClick={onClose}
    >
      <div
        style={{
          background: "var(--bg)",
          padding: 20,
          borderRadius: 12,
          width: 640,
          maxHeight: "80vh",
          overflow: "auto",
          border: "1px solid var(--border)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h3 style={{ marginTop: 0 }}>
          {existing ? "Edit rule" : "New rule"}
        </h3>
        <div style={{ marginBottom: 8 }}>
          <label style={{ fontSize: 11, color: "var(--fg-dim)" }}>Name</label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            style={{ width: "100%" }}
          />
        </div>
        <div style={{ marginBottom: 8 }}>
          <label style={{ fontSize: 11, color: "var(--fg-dim)" }}>
            Watched path
          </label>
          <div style={{ display: "flex", gap: 4 }}>
            <input
              value={watchedPath}
              onChange={(e) => setWatchedPath(e.target.value)}
              style={{ flex: 1 }}
            />
            <button onClick={pickPath}>Pick…</button>
          </div>
        </div>
        <div style={{ marginBottom: 8, display: "flex", gap: 12 }}>
          <label style={{ fontSize: 12 }}>
            <input
              type="checkbox"
              checked={enabled}
              onChange={(e) => setEnabled(e.target.checked)}
            />{" "}
            Enabled
          </label>
          <label style={{ fontSize: 12 }}>
            Priority{" "}
            <input
              type="number"
              value={priority}
              onChange={(e) => setPriority(parseInt(e.target.value || "0", 10))}
              style={{ width: 60 }}
            />
          </label>
        </div>
        <div style={{ marginBottom: 8 }}>
          <label style={{ fontSize: 11, color: "var(--fg-dim)" }}>
            Conditions (JSON, AND)
          </label>
          <textarea
            value={conditions}
            onChange={(e) => setConditions(e.target.value)}
            rows={6}
            style={{
              width: "100%",
              fontFamily: "monospace",
              fontSize: 12,
            }}
          />
        </div>
        <div style={{ marginBottom: 8 }}>
          <label style={{ fontSize: 11, color: "var(--fg-dim)" }}>
            Actions (JSON, in order)
          </label>
          <textarea
            value={actions}
            onChange={(e) => setActions(e.target.value)}
            rows={6}
            style={{
              width: "100%",
              fontFamily: "monospace",
              fontSize: 12,
            }}
          />
        </div>
        <details style={{ fontSize: 11, color: "var(--fg-dim)" }}>
          <summary>使える型</summary>
          <pre style={{ fontSize: 10 }}>{`Conditions:
  NameMatches: glob string ("*.pdf")
  ExtensionIn: ["pdf","docx"]
  MimeStartsWith: "image/"
  SizeBetween: { "min": 1000, "max": 999999 }
  ModifiedWithinDays: 7
  PathIsIn: "/Users/me/Downloads"
  HasTag: 3   // tag id

Actions:
  MoveTo: "/Users/me/Documents/{year}/{month}"
  RenameTo: "{stem}_renamed.{ext}"
  AddTag: 3
  SetColor: "#ff6b6b"
  SetRating: 5
  Notify: "message"

Template tokens: {name} {stem} {ext} {year} {month} {day}`}</pre>
        </details>
        <div
          style={{
            marginTop: 16,
            display: "flex",
            justifyContent: "flex-end",
            gap: 8,
          }}
        >
          <button onClick={onClose}>Cancel</button>
          <button className="primary" onClick={save}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
