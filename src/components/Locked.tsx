// Upgrade-CTA panel shown when the user hits a Pro feature on the free tier.

import { useEffect, useState } from "react";
import { useAppStore } from "../lib/store";
import { getLicense } from "../lib/tauri";
import type { LicenseInfo } from "../lib/types";

export function LockedPanel({
  title,
  phase,
  blurb,
  previewItems,
}: {
  title: string;
  phase: string;
  blurb: string;
  previewItems: string[];
}) {
  const setView = useAppStore((s) => s.setView);
  const [license, setLicense] = useState<LicenseInfo | null>(null);
  useEffect(() => {
    getLicense().then(setLicense).catch(console.error);
  }, []);

  const ent = license?.entitlement;
  const showLock = ent?.effective_tier === "free";

  return (
    <div className="locked">
      <div className="lock-icon">{showLock ? "🔒" : "✨"}</div>
      <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 4 }}>
        {title}
      </div>
      <div
        style={{ fontSize: 11, color: "var(--accent)", marginBottom: 12 }}
      >
        {phase}
      </div>
      <div style={{ maxWidth: 480, margin: "0 auto 16px", lineHeight: 1.5 }}>
        {blurb}
      </div>
      <ul
        style={{
          textAlign: "left",
          maxWidth: 460,
          margin: "0 auto 16px",
          paddingLeft: 18,
          color: "var(--fg-dim)",
          fontSize: 13,
        }}
      >
        {previewItems.map((item, i) => (
          <li key={i} style={{ marginBottom: 4 }}>
            {item}
          </li>
        ))}
      </ul>

      {showLock && (
        <button
          className="primary"
          style={{ padding: "10px 24px", fontSize: 14 }}
          onClick={() => setView({ kind: "pricing" })}
        >
          Pro にアップグレード
        </button>
      )}
      {ent?.source === "Trial" && (
        <p style={{ fontSize: 11, color: "var(--warn)" }}>
          トライアル残り {ent.trial_days_left} 日 — ただ動く
        </p>
      )}
    </div>
  );
}
