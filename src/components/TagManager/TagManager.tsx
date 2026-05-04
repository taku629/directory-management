import { useEffect, useState } from "react";
import { useAppStore } from "../../lib/store";
import {
  createTag,
  deleteTag,
  listFilesByTag,
  listTags,
  updateTag,
} from "../../lib/tauri";
import type { FileEntry } from "../../lib/types";
import { iconFor } from "../../lib/format";

export const TagManager = {
  View({ tagId }: { tagId: number }) {
    const tags = useAppStore((s) => s.tags);
    const setTags = useAppStore((s) => s.setTags);
    const select = useAppStore((s) => s.select);
    const [files, setFiles] = useState<FileEntry[]>([]);
    const tag = tags.find((t) => t.id === tagId);

    useEffect(() => {
      listFilesByTag(tagId).then(setFiles).catch(console.error);
    }, [tagId]);

    if (!tag) return <div className="empty">Tag not found.</div>;

    return (
      <div>
        <div className="path-bar">
          <strong>Tag:</strong> {tag.name}
          <span style={{ flex: 1 }} />
          <button
            onClick={async () => {
              const name = prompt("Rename tag:", tag.name);
              if (name && name !== tag.name) {
                await updateTag(tag.id, { name });
                setTags(await listTags());
              }
            }}
          >
            Rename
          </button>
          <button
            className="danger"
            onClick={async () => {
              if (confirm(`Delete tag "${tag.name}"? Files will be untagged.`)) {
                await deleteTag(tag.id);
                setTags(await listTags());
              }
            }}
          >
            Delete
          </button>
        </div>

        {files.length === 0 ? (
          <div className="empty">No files have this tag yet.</div>
        ) : (
          <div className="file-grid">
            {files.map((f) => (
              <div
                key={f.path}
                className="file-card"
                onClick={() => select(f)}
                title={f.path}
              >
                <div className="thumb">{iconFor(f)}</div>
                <div className="name">{f.name}</div>
              </div>
            ))}
          </div>
        )}
      </div>
    );
  },

  CreateButton() {
    const setTags = useAppStore((s) => s.setTags);
    return (
      <button
        onClick={async () => {
          const name = prompt("New tag name:");
          if (!name) return;
          await createTag(name);
          setTags(await listTags());
        }}
      >
        + New tag
      </button>
    );
  },
};
