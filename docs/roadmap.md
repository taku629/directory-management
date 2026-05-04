# Sift — Roadmap

Each phase is shipped as a numbered minor release: `0.1.x`, `0.2.x`, …

Status legend: ✅ done · 🚧 in-progress · ◻ todo

---

## Phase 1 — MVP (v0.1) ✅ scaffolded, partially implemented

**Goal: a usable local file browser with tags & search.**

### Backend
- ✅ SQLite schema (full, covers all phases)
- ✅ Connection pool (r2d2) + WAL mode
- ✅ Recursive directory scan with dedup-on-path
- ✅ FTS5 index over `name` + `note`
- ✅ Tag CRUD + file<->tag association (multi-source: user / rule / ai)
- ✅ Search with name / path-prefix / extension / tag / size / date filters
- ✅ Watched roots, favourites, smart folders (saved searches)
- ✅ File ops: move / rename / delete (with op log)
- ✅ Operation log (`operations` table populated; undo deferred)
- ◻ Background job runner (now scans are synchronous; move to Tokio)
- ◻ Thumbnail cache (sidecar JPGs in app data dir)

### Frontend
- ✅ App shell (topbar / sidebar / main / inspector / statusbar)
- ✅ File browser (grid + list, double-click to enter folders)
- ✅ Inspector with tags, rating, colour, note
- ✅ Sidebar: favorites, smart folders, tag list
- ✅ Search bar (FTS-backed)
- ✅ Settings page (watched roots, license)
- ◻ Image / video / PDF previews in inspector
- ◻ Multi-select + bulk actions
- ◻ Drag & drop into folders
- ◻ Keyboard shortcuts (j/k navigate, space preview, t tag…)
- ◻ Onboarding flow (first-run wizard)

---

## Phase 2 — Automation & cleanup (v0.2)

**Goal: auto-organise, dedup, and visualise disk usage.**

### Rules engine
- ◻ Rule schema and CRUD (✅ schema, ✅ CRUD endpoints stubbed)
- ◻ Condition evaluator (name regex, ext, size, age, tag, AI label)
- ◻ Action executor (move with template, rename, tag, set colour, run script)
- ◻ Live `notify` watcher hooked into evaluator
- ◻ Dry-run mode (preview moves before applying)
- ◻ Run-now button + scheduled runs

### Undo
- ◻ Inverse op generation for move/rename/delete/tag/untag
- ◻ Replay executor reading from `operations` table
- ◻ Time-travel UI (last 100 actions)

### Duplicates
- ◻ SHA-256 hashing background job (size-bucketed for speed)
- ◻ Perceptual hash for images (`image_hasher`)
- ◻ Cluster + present groups, with "keep oldest / newest / largest"
- ◻ Bulk move-to-Trash UI

### Disk usage
- ◻ Recursive size aggregation into a tree
- ◻ Treemap renderer (D3 or custom canvas)
- ◻ "Top N largest" + "Cold data" smart filters

### Background jobs
- ◻ Job runner with progress events
- ◻ Cancellation
- ◻ "What's running" status panel

---

## Phase 3 — AI (v0.3, Pro tier)

**Goal: tag, search, and classify intelligently.**

### Auto-tagging
- ◻ Image tagging via Claude Vision (cloud) — batch with rate-limit/cost cap
- ◻ Optional local CLIP fallback for offline mode
- ◻ Tags persisted to `ai_tags` with confidence + model
- ◻ Surface in inspector as "AI suggestions" (one-click accept)

### Classification
- ◻ User-defined buckets (Receipts, Statements, Personal, Work…)
- ◻ Few-shot prompt with examples the user has manually classified
- ◻ Confidence threshold + hand-off to rules

### Natural-language search
- ◻ NL → `SearchQuery` translator (Claude tool-use)
- ◻ Hybrid scoring: FTS match + AI-tag match + recency
- ◻ "Did you mean" suggestions

### Summarisation
- ◻ PDF / docx / md → 1-paragraph summary cached in `ai_summaries`
- ◻ Hover preview shows summary instead of file path

### OCR
- ◻ Tesseract sidecar binary (or Apple Vision on macOS)
- ◻ OCR text indexed via FTS so screenshots become searchable

### Embeddings
- ◻ Optional embedding pipeline (Voyage / OpenAI / local) → `ai_embeddings`
- ◻ "Find similar files" using cosine similarity
- ◻ Vector index (sqlite-vec or HNSW) for fast lookup

---

## Phase 4 — Cloud, team, distribution (v1.0)

**Goal: ship the paid product.**

### Licensing
- ◻ Production licensing service (Lemon Squeezy or self-hosted)
- ◻ JWT verification with bundled public key
- ◻ Online activation + offline grace period
- ◻ Per-machine activation count

### Cloud sync
- ◻ S3 / B2 (access keys)
- ◻ Dropbox / Google Drive (OAuth)
- ◻ iCloud / OneDrive (local path bridge)
- ◻ One-way snapshots → tag-merge two-way for metadata
- ◻ Conflict resolution UI

### Team
- ◻ Workspaces (multiple roots, separate tag namespaces)
- ◻ Shared tag dictionaries via cloud sync
- ◻ "Shared with me" smart folder

### Distribution
- ◻ Code signing: Apple notarisation + Windows EV cert
- ◻ Auto-updater via `tauri-plugin-updater` against own update server
- ◻ Crash reporting (Sentry or self-hosted)
- ◻ Marketing site + Stripe checkout
- ◻ Help docs site

### Polish
- ◻ Dark / light theme toggle (currently auto)
- ◻ Localisation (EN/JA at minimum)
- ◻ Accessibility pass (focus order, ARIA labels)
- ◻ "What's New" panel after updates
