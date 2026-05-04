# Sift — UI Design

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│ [Sift]  [Open Folder…]  [🔍 search ───────]      [⚙]        │ topbar
├──────────┬─────────────────────────────────┬────────────────┤
│ Library  │                                 │  Inspector     │
│  Home    │                                 │   ┌─ thumb ─┐  │
│  All     │     File browser (grid/list)    │   │         │  │
│ Favorites│                                 │   └─────────┘  │
│  …       │                                 │   name         │
│ Smart    │                                 │   path         │
│  …       │                                 │                │
│ Tags     │                                 │   tags  [+]    │
│  …       │                                 │   ★★★☆☆        │
│ Tools    │                                 │   ●●●●●        │
│  Disk    │                                 │   note ▮       │
│  Dups    │                                 │                │
│  Rules   │                                 │                │
│  AI      │                                 │                │
├──────────┴─────────────────────────────────┴────────────────┤
│  status messages …                              v0.1.0       │ statusbar
└──────────────────────────────────────────────────────────────┘
```

Layout is a CSS grid (3 cols × 3 rows). Sidebar and inspector are fixed
widths; the main panel takes the rest. Both side panels are scroll
containers.

## Views

| Sidebar entry | View | Backed by |
|---------------|------|-----------|
| Home          | Browse current folder            | `list_dir` |
| All Files     | Search across index              | `search_files` |
| Favorites     | Pinned folders                   | `list_favorites` + `list_dir` |
| Smart Folders | Saved queries                    | `list_smart_folders` + `search_files` |
| Tags          | Files with that tag              | `list_files_by_tag` |
| Disk Usage    | Treemap                          | (Phase 2) |
| Duplicates    | Groups of identical/similar      | (Phase 2) |
| Rules         | Editor + run history             | (Phase 2) |
| AI            | Suggestions + queue              | (Phase 3) |
| Settings (⚙)  | Roots, license, preferences      | mixed |

## Interactions

- **Click** a card / row → select, populate inspector
- **Double-click** folder → enter
- **Double-click** file → open with OS default (Phase 1.1)
- **Right-click** → context menu (Phase 1.1)
- **Drag** files → into folders / tags / smart folders (Phase 1.2)

## Keyboard shortcuts (planned for Phase 1.x)

| Key | Action |
|-----|--------|
| `J` / `K` | Next / previous file |
| `H` / `L` | Up to parent / Enter directory |
| `Space` | Preview overlay |
| `Enter` | Open with OS default |
| `T` | Tag input focus |
| `R` | Cycle rating |
| `1`-`5` | Set rating |
| `Cmd/Ctrl + F` | Focus search |
| `Cmd/Ctrl + Z` | Undo (Phase 2) |
| `Cmd/Ctrl + ,` | Settings |
| `?` | Show shortcuts cheatsheet |

## Design language

- Dark by default, follows OS scheme
- Single accent (`#5b9dff`) for selection / focus / primary buttons
- Colour labels reuse the standard 5: red / orange / green / blue / purple
- File icons are emoji for v0.1; replaced with proper SVG set in v0.2

## Accessibility (Phase 4)

- All interactions keyboard-reachable
- Focus ring on every focusable element
- ARIA labels on icon-only buttons
- Respects `prefers-reduced-motion`
- High-contrast theme variant

## States we render explicitly

| State | UI |
|-------|----|
| No folder open | Big "Open Folder…" prompt in main panel |
| Empty folder | "No files here" |
| No tag selected | Inspector empty state |
| File not indexed | "Index this folder" CTA before metadata edits |
| Locked feature | `🔒` panel describing the feature + "Upgrade" |
| Indexing | Progress bar in statusbar |
| Long search | Spinner inline in statusbar |
| Error | Inline error in main panel, never blocks the UI |
