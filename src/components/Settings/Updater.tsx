import { useEffect, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

type State =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "uptodate"; checkedAt: number }
  | { kind: "found"; update: Update }
  | { kind: "downloading"; downloaded: number; total: number | null }
  | { kind: "ready" }
  | { kind: "error"; message: string };

export function Updater() {
  const [state, setState] = useState<State>({ kind: "idle" });
  const [autoCheck, setAutoCheck] = useState(true);

  useEffect(() => {
    if (autoCheck) check_update();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function check_update() {
    setState({ kind: "checking" });
    try {
      const update = await check();
      if (update) {
        setState({ kind: "found", update });
      } else {
        setState({ kind: "uptodate", checkedAt: Date.now() });
      }
    } catch (e) {
      setState({ kind: "error", message: String(e) });
    }
  }

  async function install() {
    if (state.kind !== "found") return;
    let downloaded = 0;
    let total: number | null = null;
    setState({ kind: "downloading", downloaded: 0, total: null });
    try {
      await state.update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? null;
          setState({ kind: "downloading", downloaded: 0, total });
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setState({ kind: "downloading", downloaded, total });
        } else if (event.event === "Finished") {
          setState({ kind: "ready" });
        }
      });
      // tauri-plugin-process restart
      await relaunch();
    } catch (e) {
      setState({ kind: "error", message: String(e) });
    }
  }

  return (
    <section style={{ marginBottom: 24 }}>
      <h3>アップデート</h3>
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <button onClick={check_update} disabled={state.kind === "checking"}>
          {state.kind === "checking" ? "確認中…" : "今すぐ確認"}
        </button>
        <label style={{ fontSize: 12 }}>
          <input
            type="checkbox"
            checked={autoCheck}
            onChange={(e) => setAutoCheck(e.target.checked)}
          />{" "}
          起動時に自動確認
        </label>
      </div>
      <div style={{ marginTop: 8, fontSize: 13 }}>
        {state.kind === "uptodate" && (
          <span style={{ color: "var(--ok)" }}>
            最新です ({new Date(state.checkedAt).toLocaleTimeString()})
          </span>
        )}
        {state.kind === "found" && (
          <div>
            <strong>v{state.update.version}</strong> が利用可能。
            <pre
              style={{
                background: "var(--bg-3)",
                padding: 8,
                borderRadius: 4,
                fontSize: 11,
                marginTop: 4,
                whiteSpace: "pre-wrap",
              }}
            >
              {state.update.body ?? "(release notes なし)"}
            </pre>
            <button className="primary" onClick={install}>
              ダウンロード & 再起動
            </button>
          </div>
        )}
        {state.kind === "downloading" && (
          <div>
            ダウンロード中…
            {state.total ? (
              <progress
                value={state.downloaded}
                max={state.total}
                style={{ marginLeft: 8 }}
              />
            ) : (
              ` ${(state.downloaded / 1024).toFixed(0)} KB`
            )}
          </div>
        )}
        {state.kind === "ready" && <div>再起動準備中…</div>}
        {state.kind === "error" && (
          <div style={{ color: "var(--danger)" }}>{state.message}</div>
        )}
      </div>
    </section>
  );
}
