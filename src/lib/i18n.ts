// Tiny i18n. Default ja, fallback en. Locale is persisted via the backend
// `setting('locale')` so it survives restart.
//
// We do it ourselves rather than pulling react-i18next — for the UI surface
// area we have, a flat dictionary + a hook is plenty.

import { useEffect, useState } from "react";
import { getSetting, setSetting } from "./tauri";

export type Locale = "ja" | "en";

type Dict = Record<string, string>;

const ja: Dict = {
  "app.brand": "Sift",
  "topbar.openFolder": "フォルダを開く…",
  "topbar.search": "検索…",
  "sidebar.library": "ライブラリ",
  "sidebar.home": "ホーム",
  "sidebar.allFiles": "全ファイル",
  "sidebar.favorites": "お気に入り",
  "sidebar.builtin": "ビルトイン",
  "sidebar.smart": "スマートフォルダ",
  "sidebar.tags": "タグ",
  "sidebar.tools": "ツール",
  "sidebar.diskUsage": "ディスク使用量",
  "sidebar.duplicates": "重複",
  "sidebar.rules": "ルール",
  "sidebar.ai": "AI",
  "sidebar.empty": "(なし)",
  "fb.openFolderPrompt": "上の「フォルダを開く…」かサイドバーから選んで",
  "fb.noFiles": "ファイルなし",
  "fb.indexThis": "このフォルダをインデックス",
  "fb.indexing": "インデックス中…",
  "fb.list": "リスト",
  "fb.grid": "グリッド",
  "fb.delete": "削除",
  "fb.selected": "選択中",
  "preview.empty": "ファイルを選んで",
  "preview.path": "パス",
  "preview.reveal": "Finder/Explorer で表示",
  "preview.tags": "タグ",
  "preview.noTags": "タグなし",
  "preview.addTag": "タグ追加 (Enter)",
  "preview.rating": "評価",
  "preview.color": "カラーラベル",
  "preview.note": "メモ",
  "preview.ai": "AI",
  "preview.aiNeedsIndex": "AI を使うにはこのフォルダをインデックスしてから",
  "settings.title": "設定",
  "settings.watchedRoots": "監視中フォルダ",
  "settings.addRoot": "+ フォルダ追加",
  "settings.license": "ライセンス",
  "settings.locale": "言語",
  "settings.activate": "有効化",
  "settings.deactivate": "無効化",
  "settings.about": "このアプリについて",
  "rule.new": "新規ルール",
  "rule.dryRun": "ドライラン",
  "rule.runNow": "実行",
  "rule.edit": "編集",
  "rule.delete": "削除",
  "rule.watcherOn": "▶ 動作中",
  "rule.watcherOff": "⏸ 停止",
  "rule.startWatcher": "Watcher 開始",
  "rule.stopWatcher": "Watcher 停止",
  "rule.empty": "ルールまだなし",
  "rule.history": "履歴",
  "rule.noHistory": "操作履歴なし",
  "rule.undo": "Undo",
  "rule.undone": "戻し済み",
};

const en: Dict = {
  "app.brand": "Sift",
  "topbar.openFolder": "Open Folder…",
  "topbar.search": "Search…",
  "sidebar.library": "Library",
  "sidebar.home": "Home",
  "sidebar.allFiles": "All Files",
  "sidebar.favorites": "Favorites",
  "sidebar.builtin": "Built-in",
  "sidebar.smart": "Smart Folders",
  "sidebar.tags": "Tags",
  "sidebar.tools": "Tools",
  "sidebar.diskUsage": "Disk Usage",
  "sidebar.duplicates": "Duplicates",
  "sidebar.rules": "Rules",
  "sidebar.ai": "AI",
  "sidebar.empty": "(empty)",
  "fb.openFolderPrompt": "Click \"Open Folder…\" or pick from the sidebar.",
  "fb.noFiles": "No files here.",
  "fb.indexThis": "Index this folder",
  "fb.indexing": "Indexing…",
  "fb.list": "List",
  "fb.grid": "Grid",
  "fb.delete": "Delete",
  "fb.selected": "selected",
  "preview.empty": "Select a file to inspect.",
  "preview.path": "Path",
  "preview.reveal": "Reveal in Finder/Explorer",
  "preview.tags": "Tags",
  "preview.noTags": "No tags yet",
  "preview.addTag": "Add tag (Enter)",
  "preview.rating": "Rating",
  "preview.color": "Color label",
  "preview.note": "Note",
  "preview.ai": "AI",
  "preview.aiNeedsIndex": "Index this folder to enable AI buttons.",
  "settings.title": "Settings",
  "settings.watchedRoots": "Watched roots",
  "settings.addRoot": "+ Add watched folder",
  "settings.license": "License",
  "settings.locale": "Language",
  "settings.activate": "Activate",
  "settings.deactivate": "Deactivate",
  "settings.about": "About",
  "rule.new": "New rule",
  "rule.dryRun": "Dry run",
  "rule.runNow": "Run now",
  "rule.edit": "Edit",
  "rule.delete": "Delete",
  "rule.watcherOn": "▶ running",
  "rule.watcherOff": "⏸ stopped",
  "rule.startWatcher": "Start watcher",
  "rule.stopWatcher": "Stop watcher",
  "rule.empty": "No rules yet.",
  "rule.history": "History",
  "rule.noHistory": "No operation history.",
  "rule.undo": "Undo",
  "rule.undone": "undone",
};

const DICTS: Record<Locale, Dict> = { ja, en };

let currentLocale: Locale = "ja";
const subscribers = new Set<() => void>();

export function setLocale(l: Locale) {
  currentLocale = l;
  setSetting("locale", l).catch(console.error);
  subscribers.forEach((fn) => fn());
}

export function getLocale(): Locale {
  return currentLocale;
}

export function t(key: string): string {
  return DICTS[currentLocale][key] ?? DICTS.en[key] ?? key;
}

/** Bootstraps locale from the backend setting on app start. */
export async function initLocale() {
  try {
    const v = (await getSetting("locale")) as Locale | null;
    if (v === "ja" || v === "en") {
      currentLocale = v;
    }
  } catch {
    // ignore — first launch
  }
}

export function useT() {
  const [, force] = useState(0);
  useEffect(() => {
    const fn = () => force((n) => n + 1);
    subscribers.add(fn);
    return () => {
      subscribers.delete(fn);
    };
  }, []);
  return t;
}
