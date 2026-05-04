# Sift — Feature Catalogue

A flat list of every feature across all phases. Use this as the "do we have
X?" reference; for status see [roadmap.md](roadmap.md).

## Browse

- Grid view (thumbnails)
- List view (sortable columns)
- Quicklook preview on Space (Phase 1.1)
- Drag & drop into folders (Phase 1.2)
- Multi-select (Phase 1.1)
- Breadcrumb path bar
- Recently visited folders (Phase 1.1)
- Multiple watched roots
- Favourites in sidebar
- Reveal in Finder/Explorer

## Tag & metadata

- Multiple tags per file
- Coloured tags
- Hierarchical tags (parent → child)
- Star rating (0–5)
- Colour label (5 colours)
- Free-text note (FTS-indexed)
- Bulk edit metadata (Phase 1.2)
- AI suggested tags (Phase 3)

## Search

- Full-text on name + note
- Filter by tag(s)
- Filter by extension(s)
- Filter by size range
- Filter by date range
- Filter by parent path prefix
- Save as smart folder
- Natural-language search (Phase 3, Pro)
- Find similar files (Phase 3, Pro)
- OCR'd text searchable (Phase 3, Pro)

## Automate

- Rule with conditions + actions (Phase 2)
- Live folder watcher (Phase 2)
- Dry-run preview (Phase 2)
- Run on demand
- Run on schedule (Phase 2.1)
- Action: move with template `{date:Y/m}` (Phase 2)
- Action: rename with template
- Action: add tag / set colour / set rating
- Action: run shell script
- Action: notify
- Operation log + undo (Phase 2)

## Clean up

- Exact-duplicate detection (SHA-256, Phase 2)
- Similar-image detection (perceptual hash, Phase 2)
- Similar-document detection (MinHash, Phase 3)
- Disk-usage treemap (Phase 2)
- "Top 100 largest files" (Phase 2)
- "Cold files" (untouched in N months, Phase 2)
- Bulk move-to-Trash (Phase 2)

## AI (Phase 3, Pro)

- Auto-tag images (Claude Vision)
- Optional local CLIP fallback
- Auto-classify into custom buckets
- Document summarisation
- OCR (Tesseract / Apple Vision)
- Natural-language search
- "Find similar" via embeddings
- Suggest rules from your behaviour

## Cloud & team (Phase 4, Pro/Team)

- S3 / B2 / Dropbox / Google Drive / iCloud / OneDrive
- One-way snapshot sync
- Two-way metadata merge (tags, ratings)
- Conflict resolution UI
- Workspaces (Team)
- Shared tag dictionaries (Team)

## Productivity polish

- Keyboard shortcuts everywhere (Phase 1.1+)
- Dark / light theme (auto from OS)
- Localisation EN / JA (Phase 4)
- Auto-updater (Phase 4)
- Crash reporting (Phase 4, opt-in)
- Quick onboarding wizard (Phase 1.1)

## Distribution

- macOS: notarised .dmg
- Windows: signed .msi + portable .exe
- Linux: .AppImage + .deb (community-supported)
- (Future) Mac App Store, Microsoft Store, Setapp
