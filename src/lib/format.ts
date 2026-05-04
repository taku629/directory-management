import type { FileEntry } from "./types";

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)} ${units[unitIndex]}`;
}

export function iconFor(entry: FileEntry): string {
  if (entry.is_directory) return "📁";
  const ext = entry.extension ?? "";
  if (["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "heic"].includes(ext))
    return "🖼";
  if (["mp4", "mov", "mkv", "avi", "webm", "m4v"].includes(ext)) return "🎬";
  if (["mp3", "wav", "flac", "m4a", "aac", "ogg"].includes(ext)) return "🎵";
  if (["pdf"].includes(ext)) return "📕";
  if (["doc", "docx", "odt", "rtf"].includes(ext)) return "📄";
  if (["xls", "xlsx", "csv", "ods"].includes(ext)) return "📊";
  if (["ppt", "pptx", "key", "odp"].includes(ext)) return "📈";
  if (["zip", "rar", "7z", "tar", "gz", "bz2"].includes(ext)) return "🗜";
  if (
    [
      "ts",
      "tsx",
      "js",
      "jsx",
      "rs",
      "py",
      "go",
      "java",
      "c",
      "cpp",
      "h",
      "hpp",
      "rb",
      "swift",
      "kt",
    ].includes(ext)
  )
    return "💻";
  if (["md", "txt", "log"].includes(ext)) return "📝";
  return "📎";
}
