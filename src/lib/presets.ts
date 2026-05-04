// Built-in smart folders. Each maps to a SearchQuery + sort hint.

import type { PresetId } from "./store";
import type { SearchQuery } from "./types";

export const PRESETS: Record<
  PresetId,
  { label: string; icon: string; query: SearchQuery }
> = {
  biggest: {
    label: "でかいファイル Top 100",
    icon: "🐘",
    query: { size_min: 50 * 1024 * 1024, limit: 100 },
  },
  cold: {
    label: "1 年触ってないやつ",
    icon: "🧊",
    query: {
      modified_before: Math.floor(Date.now() / 1000) - 365 * 86400,
      limit: 200,
    },
  },
  recent: {
    label: "最近 1 週間",
    icon: "🆕",
    query: {
      modified_after: Math.floor(Date.now() / 1000) - 7 * 86400,
      limit: 200,
    },
  },
  images: {
    label: "画像だけ",
    icon: "🖼",
    query: {
      extensions: ["jpg", "jpeg", "png", "gif", "webp", "heic", "bmp", "tiff"],
      limit: 500,
    },
  },
  videos: {
    label: "動画だけ",
    icon: "🎬",
    query: {
      extensions: ["mp4", "mov", "mkv", "avi", "webm", "m4v"],
      limit: 200,
    },
  },
  documents: {
    label: "ドキュメント",
    icon: "📄",
    query: {
      extensions: [
        "pdf",
        "docx",
        "doc",
        "xlsx",
        "xls",
        "pptx",
        "ppt",
        "md",
        "txt",
      ],
      limit: 500,
    },
  },
};
