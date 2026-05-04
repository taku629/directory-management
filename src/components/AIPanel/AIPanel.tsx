import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  aiSearch,
  getSetting,
  setSetting,
  type AiSearchHit,
} from "../../lib/tauri";
import { useAppStore } from "../../lib/store";

export function AIPanel() {
  const [apiKey, setApiKey] = useState("");
  const [haveKey, setHaveKey] = useState<boolean | null>(null);
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<AiSearchHit[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const select = useAppStore((s) => s.select);

  useEffect(() => {
    getSetting("claude_api_key").then((v) => setHaveKey(!!v));
  }, []);

  async function saveKey() {
    if (!apiKey.trim()) return;
    await setSetting("claude_api_key", apiKey.trim());
    setApiKey("");
    setHaveKey(true);
    alert("API キー保存。AI 機能が使えるようになった。");
  }

  async function search() {
    if (!query.trim()) return;
    setBusy(true);
    setError(null);
    try {
      const r = await aiSearch(query);
      setHits(r);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div>
      <div className="path-bar">
        <strong>AI</strong>
        <span style={{ flex: 1 }} />
        <span style={{ fontSize: 11, color: "var(--fg-dim)" }}>
          Claude Sonnet 4.6
        </span>
      </div>

      {haveKey === false && (
        <div
          style={{
            border: "1px solid var(--warn)",
            borderRadius: 8,
            padding: 12,
            marginBottom: 16,
            background: "var(--bg-2)",
          }}
        >
          <p style={{ margin: 0, fontSize: 13 }}>
            AI 機能を使うには Claude API キーが必要。
            <a
              href="https://console.anthropic.com/settings/keys"
              target="_blank"
              rel="noreferrer"
              style={{ color: "var(--accent)" }}
            >
              console.anthropic.com
            </a>{" "}
            で発行して、↓ に貼る。
          </p>
          <div style={{ display: "flex", gap: 6, marginTop: 8 }}>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-ant-..."
              style={{ flex: 1 }}
            />
            <button className="primary" onClick={saveKey}>
              Save
            </button>
          </div>
          <p style={{ fontSize: 10, color: "var(--fg-dim)", marginTop: 6 }}>
            キーはローカルの SQLite に保存されるだけ。リクエストは
            api.anthropic.com に直接飛ぶ (中継サーバなし)。
          </p>
        </div>
      )}

      <h3>自然言語検索</h3>
      <div style={{ display: "flex", gap: 6 }}>
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") search();
          }}
          placeholder="例: '先月撮った犬の写真', 'PDF 大きいやつ', '昨日触った Markdown'"
          style={{ flex: 1 }}
        />
        <button className="primary" onClick={search} disabled={busy || !haveKey}>
          {busy ? "Asking…" : "Search"}
        </button>
      </div>
      <p style={{ fontSize: 12, color: "var(--fg-dim)", marginTop: 8 }}>
        Claude が tool-use で SearchQuery に変換 → ローカル検索を実行。
        実際のファイル内容は送らない。
      </p>

      {error && (
        <div className="empty" style={{ color: "var(--danger)" }}>
          {error}
        </div>
      )}

      {hits && (
        <>
          <p style={{ fontSize: 12 }}>
            <strong>{hits.length}</strong> 件
          </p>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(160px, 1fr))",
              gap: 8,
            }}
          >
            {hits.map((h) => (
              <div
                key={h.file_id}
                onClick={() =>
                  select({
                    id: h.file_id,
                    path: h.path,
                    name: h.name,
                    parent_path: "",
                    extension: null,
                    size: 0,
                    mime_type: null,
                    created_at: 0,
                    modified_at: 0,
                    is_directory: false,
                    rating: null,
                    color_label: null,
                    note: null,
                  })
                }
                className="file-card"
                title={h.path}
              >
                <img
                  src={convertFileSrc(h.path)}
                  alt={h.name}
                  loading="lazy"
                  style={{
                    width: "100%",
                    height: 80,
                    objectFit: "cover",
                    borderRadius: 4,
                    background: "#000",
                  }}
                  onError={(e) =>
                    ((e.target as HTMLImageElement).style.display = "none")
                  }
                />
                <div className="name">{h.name}</div>
              </div>
            ))}
          </div>
        </>
      )}

      <h3 style={{ marginTop: 24 }}>使い方</h3>
      <ul style={{ fontSize: 13, color: "var(--fg-dim)" }}>
        <li>画像/動画/ファイルを選んで右の inspector に AI ボタンが出る</li>
        <li>"Tag" → 画像内容から自動タグ付け (Vision)</li>
        <li>"Summarize" → 短い要約 (テキスト系のみ、PDF はまだ)</li>
        <li>"OCR" → 画像内のテキスト抽出して全文検索インデックスに混ぜ込む</li>
      </ul>
    </div>
  );
}
