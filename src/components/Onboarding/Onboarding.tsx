import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { addWatchedRoot, addFavorite, indexDirectory } from "../../lib/tauri";

const STEPS = ["intro", "pick", "indexing", "tips"] as const;
type Step = (typeof STEPS)[number];

export function Onboarding({ onClose }: { onClose: () => void }) {
  const [step, setStep] = useState<Step>("intro");
  const [picked, setPicked] = useState<string | null>(null);
  const [progress, setProgress] = useState<string>("");

  async function pick() {
    const p = await open({ directory: true });
    if (typeof p === "string") setPicked(p);
  }

  async function startIndex() {
    if (!picked) return;
    setStep("indexing");
    try {
      await addWatchedRoot(picked);
      const name = picked.split(/[\\/]/).pop() ?? picked;
      await addFavorite(picked, name);
      const r = await indexDirectory(picked);
      setProgress(`${r.indexed} ファイル取り込み完了`);
      setTimeout(() => setStep("tips"), 800);
    } catch (e) {
      setProgress(`エラー: ${e}`);
    }
  }

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.6)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 2000,
      }}
    >
      <div
        style={{
          background: "var(--bg)",
          padding: 32,
          borderRadius: 12,
          width: 520,
          border: "1px solid var(--border)",
          textAlign: "center",
        }}
      >
        {step === "intro" && (
          <>
            <h1 style={{ marginTop: 0 }}>Sift にようこそ</h1>
            <p style={{ color: "var(--fg-dim)", lineHeight: 1.6 }}>
              ローカルのファイルを <strong>整理・検索・タグ付け</strong> する
              デスクトップアプリ。
              <br />
              まずは整理したいフォルダを 1 つ指定して始めよう。
            </p>
            <div style={{ display: "flex", gap: 8, justifyContent: "center" }}>
              <button onClick={onClose}>スキップ</button>
              <button className="primary" onClick={() => setStep("pick")}>
                次へ
              </button>
            </div>
          </>
        )}

        {step === "pick" && (
          <>
            <h2 style={{ marginTop: 0 }}>フォルダを選ぶ</h2>
            <p style={{ color: "var(--fg-dim)" }}>
              ダウンロードフォルダ、写真フォルダ、Documents — お試しなので
              中身が多すぎないやつでも OK。
            </p>
            <button
              onClick={pick}
              className="primary"
              style={{ padding: "12px 24px", fontSize: 14 }}
            >
              {picked ? "別のフォルダを選ぶ" : "フォルダを選ぶ"}
            </button>
            {picked && (
              <p style={{ marginTop: 12, fontSize: 12 }}>
                選択中: <code>{picked}</code>
              </p>
            )}
            <div
              style={{
                marginTop: 16,
                display: "flex",
                gap: 8,
                justifyContent: "center",
              }}
            >
              <button onClick={() => setStep("intro")}>戻る</button>
              <button
                className="primary"
                onClick={startIndex}
                disabled={!picked}
              >
                インデックス開始
              </button>
            </div>
          </>
        )}

        {step === "indexing" && (
          <>
            <h2 style={{ marginTop: 0 }}>取り込み中…</h2>
            <p style={{ color: "var(--fg-dim)" }}>
              ファイル名と更新日時を SQLite に放り込んでいる。
              <br />
              数千〜数万ファイルなら 10 秒程度。
            </p>
            <div
              style={{
                margin: "20px auto",
                width: 60,
                height: 60,
                border: "4px solid var(--border)",
                borderTopColor: "var(--accent)",
                borderRadius: "50%",
                animation: "spin 1s linear infinite",
              }}
            />
            <p style={{ fontSize: 13 }}>{progress}</p>
            <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
          </>
        )}

        {step === "tips" && (
          <>
            <h2 style={{ marginTop: 0 }}>準備完了</h2>
            <p style={{ color: "var(--fg-dim)" }}>{progress}</p>
            <ul
              style={{
                textAlign: "left",
                lineHeight: 1.8,
                fontSize: 13,
                paddingLeft: 20,
              }}
            >
              <li>
                <strong>j / k</strong> または矢印キーで上下移動
              </li>
              <li>
                <strong>⌘ / Ctrl + クリック</strong> で複数選択
              </li>
              <li>右の inspector でタグ・評価・メモ・色付け</li>
              <li>サイドバーの「ビルトイン」でいきなり整理が始められる</li>
              <li>
                <strong>Rules</strong> でフォルダ自動整理 (Hazel 相当)
              </li>
              <li>
                <strong>AI</strong> パネルから API キー入れると自動タグ付け / 自然言語検索
              </li>
            </ul>
            <button
              className="primary"
              onClick={onClose}
              style={{ padding: "10px 24px", marginTop: 8 }}
            >
              はじめる
            </button>
          </>
        )}
      </div>
    </div>
  );
}
