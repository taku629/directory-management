# Sift

> Local folder management with auto-organization, search, tagging, and AI.
> Built as a desktop app for Mac & Windows.

Sift is a fast, lightweight desktop app that turns your local folders into a
searchable, taggable, auto-organising library. It's designed to be the one app
you open instead of Finder/Explorer when you want to actually *find* and
*organise* things — not just browse them.

## Status

Pre-alpha. Phase 1 (browse, tag, search, index) is implemented. Phases 2-4
have wired-up stubs and a fully designed schema; see
[docs/roadmap.md](docs/roadmap.md).

## Stack

- **Tauri 2** (Rust backend, OS-native WebView frontend)
- **React + TypeScript + Vite** (frontend)
- **SQLite + FTS5** (metadata, tags, search)
- **rusqlite, walkdir, notify, image** (Rust crates)

Why Tauri/Rust? Distribution size ≈10MB, low memory, fast filesystem ops.
See [docs/architecture.md](docs/architecture.md#why-tauri).

## Quick start

```bash
# 1. Install Rust toolchain (https://rustup.rs) and Node 20+
# 2. Install dependencies
npm install

# 3. Run dev (opens the desktop window with hot-reload frontend)
npm run tauri:dev

# 4. Build a release binary (.dmg / .msi / .AppImage depending on host)
npm run tauri:build
```

First launch:

1. Click **Open Folder…** in the top bar to browse a directory.
2. Click **Index this folder** to make it searchable.
3. Select a file → assign tags / rating / colour / note in the right panel.

## Repository layout

```
src/                  React frontend (TypeScript)
  components/         UI components, one folder per feature
  lib/                IPC wrapper, shared types, store, helpers
  styles/             Global CSS
src-tauri/            Tauri/Rust backend
  src/
    commands/         IPC handlers (one file per feature area)
    db/               SQLite pool + schema
    fs/               Scanner, watcher, thumbnail
    search/           Query builder + FTS execution
    rules/            Phase 2: rules engine
    duplicates/       Phase 2: hash-based dedup
    ai/               Phase 3: AI features
    cloud/            Phase 4: cloud sync
    license/          Phase 4: tier gating
docs/                 Vision, architecture, roadmap, data model, monetisation
.github/workflows/    CI
```

## Documentation

- [Vision](docs/vision.md) — what we're building and why
- [Architecture](docs/architecture.md) — how it's wired
- [Roadmap](docs/roadmap.md) — Phase 1-4 task breakdown
- [Data model](docs/data-model.md) — full SQLite schema reference
- [UI design](docs/ui-design.md) — layout, interactions, keyboard shortcuts
- [Monetization](docs/monetization.md) — free/pro/team tiers, pricing, distribution
- [Features](docs/features.md) — full feature catalogue across all phases

## License

Proprietary. See [LICENSE](LICENSE).
