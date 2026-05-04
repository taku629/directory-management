import { useEffect, useState } from "react";
import {
  activateLicense,
  addWatchedRoot,
  deactivateLicense,
  getLicense,
  listWatchedRoots,
  removeWatchedRoot,
} from "../../lib/tauri";
import type { LicenseInfo, WatchedRoot } from "../../lib/types";
import { open } from "@tauri-apps/plugin-dialog";
import { getLocale, setLocale, useT, type Locale } from "../../lib/i18n";

export function Settings() {
  const t = useT();
  const [, setLocaleState] = useState<Locale>(getLocale());
  const [license, setLicense] = useState<LicenseInfo | null>(null);
  const [roots, setRoots] = useState<WatchedRoot[]>([]);
  const [licenseKey, setLicenseKey] = useState("");

  useEffect(() => {
    getLicense().then(setLicense).catch(console.error);
    listWatchedRoots().then(setRoots).catch(console.error);
  }, []);

  async function pickRoot() {
    const picked = await open({ directory: true });
    if (typeof picked === "string") {
      await addWatchedRoot(picked);
      setRoots(await listWatchedRoots());
    }
  }

  return (
    <div style={{ maxWidth: 720 }}>
      <h2>{t("settings.title")}</h2>

      <section style={{ marginBottom: 24 }}>
        <h3>{t("settings.locale")}</h3>
        <select
          value={getLocale()}
          onChange={(e) => {
            const l = e.target.value as Locale;
            setLocale(l);
            setLocaleState(l);
          }}
        >
          <option value="ja">日本語</option>
          <option value="en">English</option>
        </select>
      </section>

      <section style={{ marginBottom: 24 }}>
        <h3>{t("settings.watchedRoots")}</h3>
        <p style={{ color: "var(--fg-dim)", fontSize: 13 }}>
          Folders that Sift indexes for full-text search and tagging. Phase 2
          will keep them in sync via a live watcher.
        </p>
        {roots.length === 0 && (
          <div className="empty">No watched roots yet.</div>
        )}
        {roots.map((r) => (
          <div
            key={r.id}
            style={{
              display: "flex",
              padding: "6px 8px",
              borderBottom: "1px solid var(--border)",
              alignItems: "center",
            }}
          >
            <span style={{ flex: 1, fontSize: 13 }}>{r.path}</span>
            <button
              className="danger"
              onClick={async () => {
                await removeWatchedRoot(r.id);
                setRoots(await listWatchedRoots());
              }}
            >
              Remove
            </button>
          </div>
        ))}
        <button onClick={pickRoot} style={{ marginTop: 8 }}>
          + Add watched folder
        </button>
      </section>

      <section style={{ marginBottom: 24 }}>
        <h3>License</h3>
        <p style={{ color: "var(--fg-dim)", fontSize: 13 }}>
          Current tier: <strong>{license?.tier ?? "…"}</strong>
        </p>
        {license?.tier === "free" ? (
          <div style={{ display: "flex", gap: 8 }}>
            <input
              placeholder="License key (try SIFT-PRO-test)"
              value={licenseKey}
              onChange={(e) => setLicenseKey(e.target.value)}
              style={{ flex: 1 }}
            />
            <button
              className="primary"
              onClick={async () => {
                try {
                  const next = await activateLicense(licenseKey);
                  setLicense(next);
                  alert(`Activated: ${next.tier}`);
                } catch (e) {
                  alert(`Activation failed: ${e}`);
                }
              }}
            >
              Activate
            </button>
          </div>
        ) : (
          <button
            className="danger"
            onClick={async () => {
              await deactivateLicense();
              setLicense(await getLicense());
            }}
          >
            Deactivate
          </button>
        )}
      </section>

      <section>
        <h3>About</h3>
        <p style={{ color: "var(--fg-dim)", fontSize: 13 }}>
          Sift v0.1.0 · See <code>docs/roadmap.md</code> for the full feature
          map.
        </p>
      </section>
    </div>
  );
}
