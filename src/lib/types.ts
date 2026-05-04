// Mirrors the Rust types serialized over Tauri IPC.

export interface FileEntry {
  id: number | null;
  path: string;
  name: string;
  parent_path: string;
  extension: string | null;
  size: number;
  mime_type: string | null;
  created_at: number;
  modified_at: number;
  is_directory: boolean;
  rating: number | null;
  color_label: string | null;
  note: string | null;
}

export interface DirListing {
  path: string;
  entries: FileEntry[];
}

export interface IndexReport {
  indexed: number;
  skipped: number;
  errors: number;
}

export interface Tag {
  id: number;
  name: string;
  color: string | null;
  parent_tag_id: number | null;
  created_at: number;
}

export interface WatchedRoot {
  id: number;
  path: string;
  enabled: boolean;
  last_scan_at: number | null;
  created_at: number;
}

export interface Favorite {
  id: number;
  path: string;
  name: string | null;
  icon: string | null;
  sort_order: number;
}

export interface SmartFolder {
  id: number;
  name: string;
  icon: string | null;
  query_json: string;
  sort_order: number;
}

export interface SearchQuery {
  text?: string;
  path_prefix?: string;
  extensions?: string[];
  tag_ids?: number[];
  size_min?: number;
  size_max?: number;
  modified_after?: number;
  modified_before?: number;
  limit?: number;
  offset?: number;
}

export interface SearchResult {
  total: number;
  items: FileEntry[];
}

export interface OperationRow {
  id: number;
  op_type: string;
  payload_json: string;
  inverse_json: string | null;
  triggered_by: string;
  created_at: number;
  undone: boolean;
}

export interface RuleInput {
  name: string;
  enabled: boolean;
  watched_path: string;
  conditions_json: string;
  actions_json: string;
  priority: number;
}

export interface PlannedOp {
  file_path: string;
  action: string;
  detail: string;
}

export interface RuleRunReport {
  matched: number;
  applied: number;
  plan: PlannedOp[];
}

export interface WatcherStatus {
  running: boolean;
  roots: string[];
}

export interface LicenseInfo {
  tier: "free" | "pro" | "team";
  activated_at: number | null;
  expires_at: number | null;
  machine_id: string | null;
}

export interface Rule {
  id: number;
  name: string;
  enabled: boolean;
  watched_path: string;
  conditions: unknown[];
  actions: unknown[];
  priority: number;
  last_run_at: number | null;
}
