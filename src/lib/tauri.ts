// Thin typed wrapper around `@tauri-apps/api/core` `invoke`.
//
// Centralising the command names here keeps refactors (and stubbing for
// tests) easy: every IPC call goes through one of the helpers below.

import { invoke } from "@tauri-apps/api/core";
import type {
  DirListing,
  Favorite,
  FileEntry,
  IndexReport,
  LicenseInfo,
  OperationRow,
  Rule,
  RuleInput,
  RuleRunReport,
  SearchQuery,
  SearchResult,
  SmartFolder,
  Tag,
  WatchedRoot,
  WatcherStatus,
} from "./types";

// ---- Phase 1: files ----
export const listDir = (path: string) =>
  invoke<DirListing>("list_dir", { path });
export const getFile = (id: number) => invoke<FileEntry>("get_file", { id });
export const indexDirectory = (path: string) =>
  invoke<IndexReport>("index_directory", { path });
export const updateFileMetadata = (
  id: number,
  patch: { rating?: number; color_label?: string; note?: string },
) => invoke<void>("update_file_metadata", { id, patch });
export const openInExplorer = (path: string) =>
  invoke<void>("open_in_explorer", { path });
export const moveFile = (src: string, dst: string) =>
  invoke<void>("move_file", { src, dst });
export const renameFile = (path: string, newName: string) =>
  invoke<string>("rename_file", { path, newName });
export const deleteFile = (path: string) =>
  invoke<void>("delete_file", { path });

// ---- Phase 1: roots / favorites / smart folders ----
export const addWatchedRoot = (path: string) =>
  invoke<number>("add_watched_root", { path });
export const listWatchedRoots = () =>
  invoke<WatchedRoot[]>("list_watched_roots");
export const removeWatchedRoot = (id: number) =>
  invoke<void>("remove_watched_root", { id });
export const addFavorite = (path: string, name?: string) =>
  invoke<number>("add_favorite", { path, name: name ?? null });
export const listFavorites = () => invoke<Favorite[]>("list_favorites");
export const removeFavorite = (id: number) =>
  invoke<void>("remove_favorite", { id });
export const createSmartFolder = (name: string, queryJson: string) =>
  invoke<number>("create_smart_folder", { name, queryJson });
export const listSmartFolders = () =>
  invoke<SmartFolder[]>("list_smart_folders");
export const deleteSmartFolder = (id: number) =>
  invoke<void>("delete_smart_folder", { id });

// ---- Phase 1: tags ----
export const createTag = (
  name: string,
  color?: string,
  parentTagId?: number,
) =>
  invoke<number>("create_tag", {
    name,
    color: color ?? null,
    parentTagId: parentTagId ?? null,
  });
export const listTags = () => invoke<Tag[]>("list_tags");
export const updateTag = (
  id: number,
  patch: { name?: string; color?: string; parent_tag_id?: number },
) =>
  invoke<void>("update_tag", {
    id,
    name: patch.name ?? null,
    color: patch.color ?? null,
    parentTagId: patch.parent_tag_id ?? null,
  });
export const deleteTag = (id: number) => invoke<void>("delete_tag", { id });
export const tagFile = (fileId: number, tagId: number, source = "user") =>
  invoke<void>("tag_file", { fileId, tagId, source });
export const untagFile = (fileId: number, tagId: number) =>
  invoke<void>("untag_file", { fileId, tagId });
export const getFileTags = (fileId: number) =>
  invoke<Tag[]>("get_file_tags", { fileId });
export const listFilesByTag = (tagId: number) =>
  invoke<FileEntry[]>("list_files_by_tag", { tagId });

// ---- Phase 1: search ----
export const searchFiles = (query: SearchQuery) =>
  invoke<SearchResult>("search_files", { query });

// ---- Phase 1: settings ----
export const getSetting = (key: string) =>
  invoke<string | null>("get_setting", { key });
export const setSetting = (key: string, value: string) =>
  invoke<void>("set_setting", { key, value });

// ---- Phase 2: rules ----
export const listRules = () => invoke<Rule[]>("list_rules");
export const createRule = (rule: RuleInput) =>
  invoke<number>("create_rule", { rule });
export const updateRule = (id: number, rule: RuleInput) =>
  invoke<void>("update_rule", { id, rule });
export const deleteRule = (id: number) => invoke<void>("delete_rule", { id });
export const runRuleNow = (id: number, dryRun: boolean) =>
  invoke<RuleRunReport>("run_rule_now", { id, dryRun });
export const startWatcher = () => invoke<WatcherStatus>("start_watcher");
export const stopWatcher = () => invoke<void>("stop_watcher");
export const watcherStatus = () => invoke<WatcherStatus>("watcher_status");

// ---- Phase 2: history ----
export const listOperations = (limit = 100) =>
  invoke<OperationRow[]>("list_operations", { limit });
export const undoOperation = (id: number) =>
  invoke<void>("undo_operation", { id });

// ---- Phase 3: AI ----
export interface AiTag {
  label: string;
  confidence: number;
}
export interface AiSearchHit {
  file_id: number;
  path: string;
  name: string;
  score: number;
  reason: string;
}
export const aiTagFile = (fileId: number) =>
  invoke<AiTag[]>("ai_tag_file", { fileId });
export const aiClassifyFile = (fileId: number, categories: string[]) =>
  invoke<string>("ai_classify_file", { fileId, categories });
export const aiSummarize = (fileId: number) =>
  invoke<string>("ai_summarize", { fileId });
export const aiSearch = (query: string) =>
  invoke<AiSearchHit[]>("ai_search", { query });
export const ocrFile = (fileId: number) =>
  invoke<string>("ocr_file", { fileId });

// ---- Phase 4: license ----
export const getLicense = () => invoke<LicenseInfo>("get_license");
export const activateLicense = (licenseKey: string) =>
  invoke<LicenseInfo>("activate_license", { licenseKey });
export const deactivateLicense = () => invoke<void>("deactivate_license");
