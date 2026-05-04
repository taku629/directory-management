import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useAppStore } from "../../lib/store";
import {
  aiSummarize,
  aiTagFile,
  createTag,
  getFileTags,
  listTags,
  ocrFile,
  openInExplorer,
  tagFile,
  untagFile,
  updateFileMetadata,
} from "../../lib/tauri";
import type { FileEntry, Tag } from "../../lib/types";
import { formatSize, iconFor } from "../../lib/format";

const COLORS = ["#ff6b6b", "#f5a623", "#4ade80", "#5b9dff", "#b388ff"];

const IMAGE_EXTS = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "heic", "svg"];
const VIDEO_EXTS = ["mp4", "mov", "webm", "m4v"];
const AUDIO_EXTS = ["mp3", "wav", "flac", "m4a", "aac", "ogg"];

function previewKind(f: FileEntry): "image" | "video" | "audio" | "text" | "none" {
  const ext = f.extension ?? "";
  if (IMAGE_EXTS.includes(ext)) return "image";
  if (VIDEO_EXTS.includes(ext)) return "video";
  if (AUDIO_EXTS.includes(ext)) return "audio";
  if (["txt", "md", "log", "json", "yaml", "yml", "toml", "csv"].includes(ext))
    return "text";
  return "none";
}

export function Preview() {
  const selected = useAppStore((s) => s.selected);
  const tags = useAppStore((s) => s.tags);
  const setTags = useAppStore((s) => s.setTags);
  const [fileTags, setFileTags] = useState<Tag[]>([]);
  const [note, setNote] = useState("");
  const [rating, setRating] = useState<number>(0);
  const [color, setColor] = useState<string | null>(null);

  useEffect(() => {
    if (!selected?.id) {
      setFileTags([]);
      setNote("");
      setRating(0);
      setColor(null);
      return;
    }
    getFileTags(selected.id).then(setFileTags).catch(console.error);
    setNote(selected.note ?? "");
    setRating(selected.rating ?? 0);
    setColor(selected.color_label ?? null);
  }, [selected]);

  if (!selected) {
    return <div className="empty">Select a file to inspect.</div>;
  }

  async function addTagByName(name: string) {
    if (!selected?.id || !name) return;
    let tag = tags.find((t) => t.name === name);
    if (!tag) {
      const newId = await createTag(name);
      const refreshed = await listTags();
      setTags(refreshed);
      tag = refreshed.find((t) => t.id === newId);
    }
    if (!tag) return;
    await tagFile(selected.id, tag.id);
    setFileTags(await getFileTags(selected.id));
  }

  async function removeTag(tagId: number) {
    if (!selected?.id) return;
    await untagFile(selected.id, tagId);
    setFileTags(await getFileTags(selected.id));
  }

  async function saveMetadata(patch: {
    rating?: number;
    color_label?: string;
    note?: string;
  }) {
    if (!selected?.id) {
      alert(
        'This file is not indexed yet — click "Index this folder" in the toolbar first.',
      );
      return;
    }
    await updateFileMetadata(selected.id, patch);
  }

  const kind = previewKind(selected);
  const src = !selected.is_directory ? convertFileSrc(selected.path) : null;

  return (
    <div>
      <div style={{ textAlign: "center", marginBottom: 12 }}>
        {kind === "image" && src ? (
          <img
            src={src}
            alt={selected.name}
            style={{
              maxWidth: "100%",
              maxHeight: 220,
              objectFit: "contain",
              borderRadius: 6,
              background: "var(--bg-3)",
            }}
          />
        ) : kind === "video" && src ? (
          <video
            src={src}
            controls
            style={{
              maxWidth: "100%",
              maxHeight: 220,
              borderRadius: 6,
              background: "#000",
            }}
          />
        ) : kind === "audio" && src ? (
          <audio src={src} controls style={{ width: "100%" }} />
        ) : (
          <div style={{ fontSize: 64 }}>{iconFor(selected)}</div>
        )}
        <div style={{ fontWeight: 600, wordBreak: "break-all", marginTop: 8 }}>
          {selected.name}
        </div>
        <div style={{ color: "var(--fg-dim)", fontSize: 11 }}>
          {selected.is_directory ? "folder" : formatSize(selected.size)}
        </div>
      </div>

      <h4>Path</h4>
      <div className="field" title={selected.path}>
        {selected.path}
      </div>
      <button onClick={() => openInExplorer(selected.path)}>
        Reveal in Finder/Explorer
      </button>

      <h4>Tags</h4>
      <div>
        {fileTags.length === 0 && (
          <span style={{ color: "var(--fg-dim)", fontSize: 12 }}>
            No tags yet
          </span>
        )}
        {fileTags.map((t) => (
          <span
            key={t.id}
            className="tag-chip"
            style={{ color: t.color ?? "var(--fg)" }}
          >
            <span className="dot" />
            <span style={{ color: "var(--fg)" }}>{t.name}</span>
            <button onClick={() => removeTag(t.id)}>×</button>
          </span>
        ))}
      </div>
      <input
        style={{ marginTop: 6, width: "100%" }}
        placeholder="Add tag (Enter)"
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            const v = (e.target as HTMLInputElement).value.trim();
            if (v) {
              addTagByName(v);
              (e.target as HTMLInputElement).value = "";
            }
          }
        }}
      />

      <h4>Rating</h4>
      <div>
        {[1, 2, 3, 4, 5].map((n) => (
          <span
            key={n}
            style={{
              cursor: "pointer",
              color: n <= rating ? "var(--warn)" : "var(--fg-dim)",
              fontSize: 18,
              marginRight: 2,
            }}
            onClick={() => {
              setRating(n);
              saveMetadata({ rating: n });
            }}
          >
            ★
          </span>
        ))}
      </div>

      <h4>Color label</h4>
      <div>
        {COLORS.map((c) => (
          <span
            key={c}
            onClick={() => {
              setColor(c);
              saveMetadata({ color_label: c });
            }}
            style={{
              display: "inline-block",
              width: 18,
              height: 18,
              background: c,
              borderRadius: 9,
              marginRight: 6,
              cursor: "pointer",
              border:
                color === c ? "2px solid var(--fg)" : "2px solid transparent",
            }}
          />
        ))}
      </div>

      <h4>Note</h4>
      <textarea
        rows={3}
        style={{ width: "100%" }}
        value={note}
        onChange={(e) => setNote(e.target.value)}
        onBlur={() => saveMetadata({ note })}
      />

      <AiActions selected={selected} kind={kind} onTagsChanged={async () => {
        if (selected.id) setFileTags(await getFileTags(selected.id));
      }} />
    </div>
  );
}

function AiActions({
  selected,
  kind,
  onTagsChanged,
}: {
  selected: FileEntry;
  kind: "image" | "video" | "audio" | "text" | "none";
  onTagsChanged: () => void;
}) {
  const [busy, setBusy] = useState<string | null>(null);
  const [result, setResult] = useState<string | null>(null);

  if (!selected.id) {
    return (
      <>
        <h4>AI</h4>
        <p style={{ fontSize: 11, color: "var(--fg-dim)" }}>
          このフォルダをインデックスしてから AI ボタンが使える。
        </p>
      </>
    );
  }

  async function run(label: string, fn: () => Promise<unknown>) {
    setBusy(label);
    setResult(null);
    try {
      const r = await fn();
      setResult(typeof r === "string" ? r : JSON.stringify(r, null, 2));
      onTagsChanged();
    } catch (e) {
      setResult(`Error: ${e}`);
    } finally {
      setBusy(null);
    }
  }

  return (
    <>
      <h4>AI</h4>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
        {kind === "image" && (
          <>
            <button
              onClick={() =>
                selected.id != null && run("tag", () => aiTagFile(selected.id!))
              }
              disabled={busy !== null}
            >
              {busy === "tag" ? "…" : "Tag"}
            </button>
            <button
              onClick={() =>
                selected.id != null && run("ocr", () => ocrFile(selected.id!))
              }
              disabled={busy !== null}
            >
              {busy === "ocr" ? "…" : "OCR"}
            </button>
          </>
        )}
        {kind === "text" && (
          <button
            onClick={() =>
              selected.id != null &&
              run("sum", () => aiSummarize(selected.id!))
            }
            disabled={busy !== null}
          >
            {busy === "sum" ? "…" : "Summarize"}
          </button>
        )}
      </div>
      {kind === "none" && (
        <p style={{ fontSize: 11, color: "var(--fg-dim)" }}>
          画像/テキスト系で AI 機能が使える。
        </p>
      )}
      {result && (
        <pre
          style={{
            marginTop: 8,
            fontSize: 11,
            background: "var(--bg-3)",
            padding: 8,
            borderRadius: 4,
            whiteSpace: "pre-wrap",
            maxHeight: 200,
            overflow: "auto",
          }}
        >
          {result}
        </pre>
      )}
    </>
  );
}
