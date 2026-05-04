import { create } from "zustand";
import type { FileEntry, Tag } from "./types";

type View =
  | { kind: "browse"; path: string }
  | { kind: "tag"; tagId: number }
  | { kind: "smart"; smartFolderId: number }
  | { kind: "search"; text: string }
  | { kind: "duplicates" }
  | { kind: "disk-usage" }
  | { kind: "rules" }
  | { kind: "ai" }
  | { kind: "settings" };

interface AppStore {
  view: View;
  setView: (v: View) => void;

  selected: FileEntry | null;
  select: (f: FileEntry | null) => void;

  tags: Tag[];
  setTags: (t: Tag[]) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  view: { kind: "browse", path: defaultHome() },
  setView: (view) => set({ view }),
  selected: null,
  select: (selected) => set({ selected }),
  tags: [],
  setTags: (tags) => set({ tags }),
}));

function defaultHome(): string {
  // Prefer `~`/`%USERPROFILE%`-like path; the backend handles real expansion.
  // The user typically picks a folder via the Open button on first launch.
  return "";
}
