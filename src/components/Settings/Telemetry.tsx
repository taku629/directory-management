// Opt-in telemetry. Off by default. Sends anonymous feature counters.
//
// Privacy contract:
//   - never sends file names, paths, content, tags, license keys, API keys
//   - only sends: app version, OS, locale, anonymous machine id (hash),
//                 feature usage counters (e.g. `rules:run`, `ai:tag` +1)
//   - one-line ping every 24h while the app is open

import { useEffect, useState } from "react";
import { getSetting, setSetting } from "../../lib/tauri";

export function Telemetry() {
  const [enabled, setEnabled] = useState<boolean | null>(null);

  useEffect(() => {
    getSetting("telemetry_enabled").then((v) => setEnabled(v === "1"));
  }, []);

  async function toggle(v: boolean) {
    setEnabled(v);
    await setSetting("telemetry_enabled", v ? "1" : "0");
  }

  if (enabled === null) return null;

  return (
    <section style={{ marginBottom: 24 }}>
      <h3>使用状況の送信</h3>
      <label style={{ fontSize: 13 }}>
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => toggle(e.target.checked)}
        />{" "}
        匿名の機能利用カウンタを送る (オフがデフォルト)
      </label>
      <p style={{ fontSize: 11, color: "var(--fg-dim)", marginTop: 4 }}>
        送るのは「アプリのバージョン / OS / ロケール / どの機能を何回使ったか」
        だけ。ファイル名・パス・タグ・ファイル内容・API キー・ライセンスキーは
        絶対に送らない。
      </p>
    </section>
  );
}
