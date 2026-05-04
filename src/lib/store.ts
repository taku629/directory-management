import { create } from "zustand";
import type { FileEntry, Tag } from "./types";

type View =
  | { kind: "browse"; path: string }
  | { kind: "tag"; tagId: number }
  | { kind: "smart"; smartFolderId: number }
  | { kind: "search"; text: string }
  | { kind: "preset"; preset: PresetId }
  | { kind: "duplicates" }
  | { kind: "disk-usage" }
  | { kind: "rules" }
  | { kind: "ai" }
  | { kind: "settings" };

export type PresetId =
  | "biggest"
  | "cold"
  | "recent"
  | "images"
  | "videos"
  | "documents";

interface AppStore {
  view: View;
  setView: (v: View) => void;

  /** Inspector focus — the most recently clicked file. */
  selected: FileEntry | null;
  select: (f: FileEntry | null) => void;

  /** Multi-select set, keyed by absolute path. */
  selectedPaths: Set<string>;
  toggleSelected: (path: string) => void;
  setSelection: (paths: string[]) => void;
  clearSelection: () => void;

  tags: Tag[];
  setTags: (t: Tag[]) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  view: { kind: "browse", path: "" },
  setView: (view) => set({ view, selectedPaths: new Set(), selected: null }),

  selected: null,
  select: (selected) =>
    set({
      selected,
      selectedPaths: selected ? new Set([selected.path]) : new Set(),
    }),

  selectedPaths: new Set<string>(),
  toggleSelected: (path) =>
    set((s) => {
      const next = new Set(s.selectedPaths);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return { selectedPaths: next };
    }),
  setSelection: (paths) => set({ selectedPaths: new Set(paths) }),
  clearSelection: () => set({ selectedPaths: new Set() }),

  tags: [],
  setTags: (tags) => set({ tags }),
}));
