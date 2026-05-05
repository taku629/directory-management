// Top-bar pill that shows trial status and links to pricing.

import { useEffect, useState } from "react";
import { getLicense } from "../lib/tauri";
import type { Entitlement } from "../lib/types";

export function TrialBadge({ onUpgrade }: { onUpgrade: () => void }) {
  const [ent, setEnt] = useState<Entitlement | null>(null);

  useEffect(() => {
    let mounted = true;
    async function refresh() {
      try {
        const lic = await getLicense();
        if (mounted) setEnt(lic.entitlement);
      } catch {}
    }
    refresh();
    const id = setInterval(refresh, 60_000);
    return () => {
      mounted = false;
      clearInterval(id);
    };
  }, []);

  if (!ent) return null;
  if (ent.source === "License") {
    return (
      <span
        style={{
          padding: "2px 8px",
          background: "var(--ok)",
          color: "#000",
          borderRadius: 999,
          fontSize: 10,
          fontWeight: 600,
        }}
      >
        {ent.effective_tier.toUpperCase()}
      </span>
    );
  }
  if (ent.source === "Trial") {
    return (
      <button
        onClick={onUpgrade}
        style={{
          padding: "2px 10px",
          background: "var(--warn)",
          color: "#000",
          border: "none",
          borderRadius: 999,
          fontSize: 11,
          fontWeight: 600,
          cursor: "pointer",
        }}
      >
        トライアル残 {ent.trial_days_left}日
      </button>
    );
  }
  return (
    <button
      onClick={onUpgrade}
      style={{
        padding: "2px 10px",
        background: "var(--accent)",
        color: "#fff",
        border: "none",
        borderRadius: 999,
        fontSize: 11,
        fontWeight: 600,
        cursor: "pointer",
      }}
    >
      Pro にする
    </button>
  );
}
