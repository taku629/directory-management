import { useEffect, useState } from "react";
import { activateLicense, getLicense } from "../../lib/tauri";
import type { LicenseInfo } from "../../lib/types";
import { invoke } from "@tauri-apps/api/core";

interface Tier {
  id: "free" | "pro" | "team";
  label: string;
  price: string;
  per: string;
  cta: string;
  buyUrl?: string;
  features: string[];
  highlight?: boolean;
}

// Lemon Squeezy checkout URLs. Set these at build time:
//   VITE_BUY_URL  = https://<store>.lemonsqueezy.com/buy/<pro-product-uuid>
//   VITE_TEAM_URL = https://<store>.lemonsqueezy.com/buy/<team-product-uuid>
// Until those products exist, both fall back to the store front page.
const BUY_URL = import.meta.env.VITE_BUY_URL ?? "https://taku629.lemonsqueezy.com";
const TEAM_URL = import.meta.env.VITE_TEAM_URL ?? "https://taku629.lemonsqueezy.com";

const TIERS: Tier[] = [
  {
    id: "free",
    label: "Free",
    price: "¥0",
    per: "ずっと",
    cta: "今これ",
    features: [
      "ブラウズ・タグ・検索",
      "スマートフォルダ (プリセット & 自作)",
      "完全一致の重複検出",
      "ディスク使用量",
      "Watched root 1 個",
    ],
  },
  {
    id: "pro",
    label: "Pro",
    price: "¥4,800",
    per: "買い切り (or ¥500/月)",
    cta: "Pro を買う",
    highlight: true,
    buyUrl: BUY_URL,
    features: [
      "Free のすべて",
      "ルール自動整理 + ファイル監視",
      "AI 自動タグ付け / 自然言語検索 / 要約 / OCR",
      "類似画像検出 (perceptual hash)",
      "Watched root 無制限",
      "クラウド同期 (S3 / Dropbox / iCloud)",
      "1 年間無料アップデート",
    ],
  },
  {
    id: "team",
    label: "Team",
    price: "¥1,200",
    per: "/ ユーザ / 月",
    cta: "問い合わせ",
    buyUrl: TEAM_URL,
    features: [
      "Pro のすべて",
      "ワークスペース (複数ライブラリ)",
      "共有タグ辞書",
      "優先サポート",
    ],
  },
];

export function Pricing() {
  const [license, setLicense] = useState<LicenseInfo | null>(null);
  const [key, setKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getLicense().then(setLicense).catch(console.error);
  }, []);

  async function activate() {
    setBusy(true);
    setError(null);
    try {
      const info = await activateLicense(key);
      setLicense(info);
      setKey("");
      alert("有効化完了 🎉");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function openExternal(url: string) {
    try {
      await invoke("plugin:opener|open_url", { url });
    } catch {
      // Fallback: pop a window in the WebView
      window.open(url, "_blank");
    }
  }

  const ent = license?.entitlement;

  return (
    <div style={{ maxWidth: 1000, margin: "0 auto" }}>
      <h2 style={{ textAlign: "center" }}>料金</h2>

      {ent && (
        <div
          style={{
            textAlign: "center",
            padding: 12,
            background: "var(--bg-2)",
            border: "1px solid var(--border)",
            borderRadius: 8,
            marginBottom: 24,
          }}
        >
          現在のステータス: <strong>{ent.effective_tier.toUpperCase()}</strong>
          {ent.source === "Trial" && ent.trial_days_left != null && (
            <span style={{ marginLeft: 8, color: "var(--warn)" }}>
              トライアル残り {ent.trial_days_left} 日
            </span>
          )}
          {ent.source === "License" && (
            <span style={{ marginLeft: 8, color: "var(--ok)" }}>
              ライセンス有効
            </span>
          )}
        </div>
      )}

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))",
          gap: 16,
        }}
      >
        {TIERS.map((t) => (
          <div
            key={t.id}
            style={{
              border: t.highlight
                ? "2px solid var(--accent)"
                : "1px solid var(--border)",
              borderRadius: 12,
              padding: 20,
              background: t.highlight ? "var(--bg-2)" : "var(--bg)",
              position: "relative",
            }}
          >
            {t.highlight && (
              <div
                style={{
                  position: "absolute",
                  top: -10,
                  right: 16,
                  background: "var(--accent)",
                  color: "white",
                  padding: "2px 8px",
                  borderRadius: 4,
                  fontSize: 10,
                  fontWeight: 600,
                }}
              >
                おすすめ
              </div>
            )}
            <h3 style={{ marginTop: 0 }}>{t.label}</h3>
            <div style={{ fontSize: 28, fontWeight: 700 }}>{t.price}</div>
            <div style={{ fontSize: 11, color: "var(--fg-dim)" }}>{t.per}</div>
            <ul
              style={{
                paddingLeft: 18,
                fontSize: 13,
                marginTop: 16,
                lineHeight: 1.7,
              }}
            >
              {t.features.map((f, i) => (
                <li key={i}>{f}</li>
              ))}
            </ul>
            <button
              className={t.highlight ? "primary" : ""}
              style={{ width: "100%", padding: "10px", marginTop: 12 }}
              disabled={ent?.effective_tier === t.id || !t.buyUrl}
              onClick={() => t.buyUrl && openExternal(t.buyUrl)}
            >
              {ent?.effective_tier === t.id ? "現在のプラン" : t.cta}
            </button>
          </div>
        ))}
      </div>

      <h3 style={{ marginTop: 32 }}>ライセンスを有効化</h3>
      <p style={{ color: "var(--fg-dim)", fontSize: 13 }}>
        購入後にメールで届くキーをここに貼って Activate。
      </p>
      <div style={{ display: "flex", gap: 8 }}>
        <input
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder="SIFT-PRO-xxxx... もしくは署名済みトークン"
          style={{ flex: 1 }}
        />
        <button className="primary" onClick={activate} disabled={busy}>
          {busy ? "確認中…" : "Activate"}
        </button>
      </div>
      {error && (
        <div style={{ color: "var(--danger)", marginTop: 8, fontSize: 12 }}>
          {error}
        </div>
      )}
      <p style={{ fontSize: 11, color: "var(--fg-dim)", marginTop: 16 }}>
        マシン ID: <code>{license?.machine_id}</code>
      </p>
    </div>
  );
}
