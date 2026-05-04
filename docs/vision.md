# Sift — Vision

## One-liner

**Sift turns your local folders into a fast, searchable, self-organising
library.**

## The problem

Finder and Explorer are 30-year-old metaphors. Most people's machines have
50k–500k files: photos auto-saving from a phone, downloads piling up,
screenshots, AI-generated assets, work documents in 8 different cloud roots.
Search is slow, tags are non-existent or ignored, and "organisation" means
manually dragging files into folders that never quite work.

There are point tools for each pain (Hazel for rules, DaisyDisk for usage,
Eagle for assets, Czkawka for duplicates) but nothing that ties them together
with a fast browser and AI. Eagle is the closest, but is heavy (Electron),
focused on creative assets, and has no automation/AI story.

## The product

A single desktop app that:

1. **Indexes** any folders you point at, into a fast local DB (SQLite + FTS).
2. **Browses** them with a modern UI (grid/list/preview, like Eagle).
3. **Tags** files with user tags + AI-suggested tags + colour labels + ratings.
4. **Searches** them by name, content, tags, metadata, or natural language.
5. **Automates** organisation via Hazel-style rules + a live folder watcher.
6. **Cleans up** with duplicate detection and disk-usage visualisation.
7. **Augments** with AI: auto-tag images, summarise documents, NL search, OCR.
8. **Syncs** tags & metadata to the cloud (S3/Dropbox/iCloud) — opt-in.

## Target users

| Segment | Pain | Why Sift |
|---------|------|----------|
| Knowledge workers | Downloads chaos, can't find documents | Rules + AI search |
| Designers & creators | Asset management, mood boards | Tags + thumbnails + smart folders |
| Photographers | Triage thousands of RAW + JPG | Rating + colour + dedup |
| Developers | Repos + assets + screenshots scattered | Watch folders + tags + fast search |
| ML engineers | Datasets, model checkpoints | Disk usage + dedup + AI tagging |

## Differentiation

- **Tauri-native**: 10MB install, low memory, fast — vs Eagle/Photos heavy
- **Local-first AI**: Claude Vision optional; nothing is forced to the cloud
- **One app, full stack**: rules + dedup + disk usage + AI in one place
- **Cross-platform from day one**: same UX on Mac and Windows

## Non-goals (for now)

- Mobile clients (web companion may come later)
- Shared multi-user editing
- Replacing your cloud storage — we *enhance* it, not replace it
- Becoming a full IDE / code editor

## What "done" looks like

- A user installs Sift, points it at `~/Downloads` and `~/Pictures`.
- Within 30s, indexing is complete and they can search by name.
- They set up one rule: "PDFs from Downloads → Documents/{year}/{month}".
- They activate Pro, run AI auto-tagging on Pictures overnight.
- Next day, they search "screenshots of the dashboard" and it just works.
- They tell their friends.
