# Sift — Architecture

## Tech choices

### Why Tauri

| | Tauri | Electron |
|-|-------|----------|
| Install size | 5–15 MB | 100–200 MB |
| RAM at idle | 30–80 MB | 200–500 MB |
| File ops speed | Native Rust | Node.js |
| Code signing & bundling | Built-in (`tauri build`) | Add-on (`electron-builder`) |
| Updater | Built-in (`tauri-plugin-updater`) | Add-on |
| Frontend stack | React + Vite (any web stack) | Same |
| Maturity | 2.x stable | mature |

For a paid productivity app, **install size and RAM matter** — they're the
top complaints in App Store reviews of competitors. Rust also wins by an order
of magnitude on the workload Sift is built for: scanning, hashing,
thumbnailing tens of thousands of files.

### Why SQLite + FTS5

- Embedded, zero-config, single file
- FTS5 is fast and built into rusqlite
- WAL mode = concurrent reads while indexer writes
- Ships with rusqlite via the `bundled` feature → no system SQLite needed

### Why React (not Svelte/Solid)

- Largest ecosystem (date pickers, tree views, virtualisation, etc.)
- Easier to hire / find contributors
- Tradeoff is bundle size, but it's all WebView so cost is trivial

## Process model

```
┌──────────────────────────────────────────────────────────┐
│                     OS WebView                           │
│  React UI ←→ @tauri-apps/api `invoke('command', args)`   │
└────────────────────────┬─────────────────────────────────┘
                         │  IPC  (typed JSON)
┌────────────────────────▼─────────────────────────────────┐
│  Tauri Rust process (one main process)                   │
│   ├─ AppState (DB pool, watcher handle)                  │
│   ├─ commands::*  (one file per feature area)            │
│   ├─ fs::scanner / watcher / thumbnail                   │
│   ├─ search::indexer                                     │
│   ├─ rules / duplicates / ai / cloud / license           │
│   └─ rusqlite + r2d2  →  SQLite DB                       │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
                ~/.local/share/sift/sift.db   (or platform equivalent)
```

## Module boundaries

| Module | Responsibility | Phase |
|--------|---------------|-------|
| `commands::files`     | Browse, index, metadata, file ops | 1 |
| `commands::tags`      | Tag CRUD + file<->tag linking | 1 |
| `commands::search`    | Query execution | 1 |
| `commands::settings`  | KV settings | 1 |
| `commands::rules`     | Rule CRUD; engine in `rules::` | 2 |
| `commands::duplicates`| Dedup queries; finders in `duplicates::` | 2 |
| `commands::disk`      | Disk treemap | 2 |
| `commands::history`   | Operation log + undo | 2 |
| `commands::ai`        | AI tagging, classification, NL search | 3 |
| `commands::cloud`     | Provider config + sync | 4 |
| `commands::license`   | Activation + tier gating | 4 |

Each `commands::X` module *only* coordinates IPC and DB; logic lives in the
matching domain module (`X::`). This keeps commands thin and units testable.

## Concurrency model

Tauri commands are async by default. For Phase 1 we use **synchronous**
rusqlite calls inside short-lived commands (≤10ms typical). Long-running work
gets spawned onto Tokio:

- Indexing a directory → `tokio::task::spawn_blocking` returns a job id.
- Watcher debounces FS events → triggers reindex on a job queue.
- Hashing for dedup → background pool, throttled to N=4 cores by default.

Progress is reported back to the UI via Tauri events
(`window.emit("scan:progress", { done, total })`).

## Data flow: indexing a folder

1. UI calls `index_directory(path)`.
2. Backend opens a transaction, walks the directory with `walkdir`.
3. For each entry, upserts into `files` and refreshes `files_fts`.
4. Returns a `ScanReport` (counts).
5. Phase 2: a watcher is registered on the path; subsequent events trigger
   incremental upserts and rule evaluation.

## Tier gating

Every privileged command calls `license::require_tier(state, Tier::Pro)`
before doing real work. The free tier returns a typed `LockedFeature` error
the UI renders as the "🔒 Upgrade" panel. This keeps the upgrade path
discoverable from inside features (not buried behind a separate paywall).

## Extending the app

To add a new feature area:

1. Create `src-tauri/src/<area>/mod.rs` with domain types & logic.
2. Create `src-tauri/src/commands/<area>.rs` with `#[tauri::command]` fns.
3. Register the commands in `lib.rs`'s `invoke_handler!`.
4. Add a typed wrapper in `src/lib/tauri.ts`.
5. Add a UI component under `src/components/<Area>/`.
6. (Optional) wire up a sidebar entry in `Sidebar.tsx`.
