//! Full SQLite schema covering Phase 1-4.
//!
//! Schema is created idempotently on first run. Future migrations should be
//! added as `ALTER TABLE` statements gated by a `schema_version` row in
//! `settings`.

pub const SCHEMA_SQL: &str = r#"
-- ---------------------------------------------------------------------------
-- Phase 1: core tables
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    parent_path TEXT NOT NULL,
    extension TEXT,
    size INTEGER NOT NULL,
    mime_type TEXT,
    created_at INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    accessed_at INTEGER,
    indexed_at INTEGER NOT NULL,
    is_directory INTEGER NOT NULL DEFAULT 0,
    hash_sha256 TEXT,
    hash_phash TEXT,
    rating INTEGER,
    color_label TEXT,
    note TEXT,
    deleted INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_files_parent     ON files(parent_path);
CREATE INDEX IF NOT EXISTS idx_files_extension  ON files(extension);
CREATE INDEX IF NOT EXISTS idx_files_hash       ON files(hash_sha256);
CREATE INDEX IF NOT EXISTS idx_files_phash      ON files(hash_phash);
CREATE INDEX IF NOT EXISTS idx_files_modified   ON files(modified_at);
CREATE INDEX IF NOT EXISTS idx_files_name       ON files(name);

-- Full-text search over file name + note.
-- Phase 3 OCR/AI text is appended via separate triggers.
CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
    name, note, content='', tokenize='unicode61'
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    parent_tag_id INTEGER REFERENCES tags(id) ON DELETE SET NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS file_tags (
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    source TEXT NOT NULL DEFAULT 'user',
    created_at INTEGER NOT NULL,
    PRIMARY KEY (file_id, tag_id)
);
CREATE INDEX IF NOT EXISTS idx_file_tags_tag ON file_tags(tag_id);

CREATE TABLE IF NOT EXISTS watched_roots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_scan_at INTEGER,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS smart_folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    icon TEXT,
    query_json TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS favorites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL,
    name TEXT,
    icon TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- ---------------------------------------------------------------------------
-- Phase 2: automation, undo
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    watched_path TEXT NOT NULL,
    conditions_json TEXT NOT NULL,
    actions_json TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    last_run_at INTEGER,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_rules_path ON rules(watched_path);

CREATE TABLE IF NOT EXISTS operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    op_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    inverse_json TEXT,
    triggered_by TEXT NOT NULL DEFAULT 'user',
    created_at INTEGER NOT NULL,
    undone INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_operations_created ON operations(created_at DESC);

-- ---------------------------------------------------------------------------
-- Phase 3: AI augmentation
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS ai_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    confidence REAL NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ai_tags_file  ON ai_tags(file_id);
CREATE INDEX IF NOT EXISTS idx_ai_tags_label ON ai_tags(label);

CREATE TABLE IF NOT EXISTS ai_embeddings (
    file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
    model TEXT NOT NULL,
    dim INTEGER NOT NULL,
    embedding BLOB NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_summaries (
    file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
    summary TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ocr_text (
    file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    lang TEXT,
    created_at INTEGER NOT NULL
);

-- ---------------------------------------------------------------------------
-- Phase 4: licensing, cloud, team
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS license (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    license_key TEXT,
    tier TEXT NOT NULL DEFAULT 'free',
    activated_at INTEGER,
    expires_at INTEGER,
    machine_id TEXT
);
INSERT OR IGNORE INTO license (id, tier) VALUES (1, 'free');

CREATE TABLE IF NOT EXISTS cloud_sync (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL,
    config_json TEXT NOT NULL,
    last_sync_at INTEGER,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS workspaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    config_json TEXT,
    created_at INTEGER NOT NULL
);
"#;
