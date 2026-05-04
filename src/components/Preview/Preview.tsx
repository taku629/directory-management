import { useEffect, useState } from "react";
import { useAppStore } from "../../lib/store";
import {
  createTag,
  getFileTags,
  listTags,
  openInExplorer,
  tagFile,
  untagFile,
  updateFileMetadata,
} from "../../lib/tauri";
import type { Tag } from "../../lib/types";
import { formatSize, iconFor } from "../../lib/format";

const COLORS = ["#ff6b6b", "#f5a623", "#4ade80", "#5b9dff", "#b388ff"];

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

  return (
    <div>
      <div style={{ textAlign: "center", marginBottom: 12 }}>
        <div style={{ fontSize: 64 }}>{iconFor(selected)}</div>
        <div style={{ fontWeight: 600, wordBreak: "break-all" }}>
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
    </div>
  );
}
