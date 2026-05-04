# Sift — Data Model

The full schema lives in
[`src-tauri/src/db/schema.rs`](../src-tauri/src/db/schema.rs). This doc
describes the tables and their relationships at a higher level.

## Files

```
files
  id (PK)
  path TEXT UNIQUE          ← canonical absolute path
  name, parent_path, extension, size, mime_type
  created_at, modified_at, accessed_at, indexed_at
  is_directory
  hash_sha256, hash_phash    ← Phase 2 (background job)
  rating (0-5), color_label, note  ← user metadata
  deleted (soft-delete flag)
```

The same row is reused across phases — Phase 2 fills in hashes, Phase 3 joins
to `ai_tags`/`ai_embeddings`/`ocr_text`/`ai_summaries`.

`files_fts` is a separate FTS5 virtual table over `name + note`. Phase 3 OCR
and AI summaries are concatenated into the FTS row so a single search query
hits all text sources.

## Tags

```
tags                 ← user-managed taxonomy
  id, name UNIQUE, color, parent_tag_id, created_at
file_tags
  (file_id, tag_id) PK
  source TEXT        ← 'user' | 'rule' | 'ai'
  created_at
```

Multi-source design lets us show "AI suggested" vs "you tagged" differently
in the UI, and lets the user purge AI tags without touching their own.

## Library structure

```
watched_roots        ← folders Sift indexes (multi-root supported)
favorites            ← sidebar pinned folders
smart_folders        ← saved searches (query_json is a SearchQuery)
settings             ← simple key-value config
```

## Automation

```
rules                ← Phase 2
  watched_path
  conditions_json    ← Vec<Condition>
  actions_json       ← Vec<Action>
  priority, last_run_at
operations           ← every destructive op logged here
  op_type            ← 'move' | 'rename' | 'delete' | 'tag_add' | …
  payload_json
  inverse_json       ← used by undo
  triggered_by       ← 'user' | 'rule:<id>'
  undone
```

The `operations` table is the audit trail. Phase 2 adds the inverse-op
generator and the undo executor; the table is already populated in Phase 1
for forward compatibility.

## AI

```
ai_tags              ← Phase 3: per-file labels with confidence + model
ai_embeddings        ← Phase 3: dense vectors for semantic similarity
ai_summaries         ← Phase 3: short summary text for documents
ocr_text             ← Phase 3: text extracted from images / PDFs
```

## Cloud / team / licensing

```
license              ← single-row table; tier ∈ {free, pro, team}
cloud_sync           ← Phase 4: provider configs (S3/Dropbox/…)
workspaces           ← Phase 4: team mode multi-config
```

## Migrations

The schema is created idempotently with `CREATE TABLE IF NOT EXISTS` so
fresh installs and existing databases both bootstrap cleanly. Future schema
changes will use a `schema_version` row in `settings` and gated `ALTER TABLE`
blocks; we keep migrations forward-only.

## Indexes

The schema declares the indexes we know we'll need from query patterns:

| Index | Used by |
|-------|---------|
| `idx_files_parent`     | folder browser |
| `idx_files_extension`  | extension filter |
| `idx_files_hash`       | dedup |
| `idx_files_phash`      | similar images |
| `idx_files_modified`   | recency sort |
| `idx_files_name`       | rename lookup |
| `idx_file_tags_tag`    | "files with tag X" |
| `idx_rules_path`       | watcher dispatch |
| `idx_operations_created` | history view |
| `idx_ai_tags_file/label` | AI search |

## Storage location

`AppState::init` resolves Tauri's `app_data_dir()`:

| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/dev.taku629.sift/` |
| Windows | `%APPDATA%\dev.taku629.sift\` |
| Linux | `~/.local/share/dev.taku629.sift/` |

The DB lives at `<data_dir>/sift.db`. Thumbnails (Phase 1.x) cache to
`<data_dir>/thumbs/`. Cloud sync state (Phase 4) to `<data_dir>/cloud/`.
