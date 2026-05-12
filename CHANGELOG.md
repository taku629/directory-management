# Changelog

All notable changes to Sift are documented here.
This project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added since 0.1.0 snapshot
- App icon / logo mark (a stylised sieve) — `src-tauri/icons/icon.svg` is the
  source; the raster set was regenerated with `tauri icon`. Still a clean
  placeholder, not final art.
- `licenser/` now has a working `POST /webhook/lemonsqueezy` (HMAC-SHA256
  verification, issues a token on purchase, revokes on refund/cancel), `POST
  /revoke`, device-activation tracking with a per-license limit, and optional
  Resend email delivery.
- Pricing checkout URLs are configurable via `VITE_BUY_URL` / `VITE_TEAM_URL`.

### Pending before first public release
- Generate the Tauri updater signing keypair and the license Ed25519 keypair;
  wire public keys into `tauri.conf.json` / build env (see `docs/distribution.md`).
- Create the Lemon Squeezy store + Pro/Team products; set `VITE_BUY_URL` /
  `VITE_TEAM_URL` at build time and the webhook secret for `licenser/`.
- Final icon/logo art, app screenshots, `landing/og.png`.
- Apple notarization + Windows code-signing certificates.
- Legal review of `landing/privacy.html` and `landing/terms.html`.

## [0.1.0] — unreleased

First feature-complete build. Codename "Sift".

### Added
- **Library**: recursive folder indexing into SQLite, grid/list browser,
  tags (multi, colored, hierarchical), star ratings, color labels, notes.
- **Search**: full-text search over file names and notes (FTS5), favorites,
  smart folders (saved searches) with built-in presets.
- **Automation**: rule engine with structured condition/action editor,
  glob matching, filename templating; filesystem watcher (debounced).
- **Cleanup**: exact-match duplicate detection (SHA-256, size-bucketed),
  perceptual-hash similar-image detection, disk-usage treemap.
- **AI** (Pro): Claude API integration — image auto-tagging (Vision),
  natural-language search, document summaries, OCR; results cached in SQLite.
- **Safety**: undo for move/rename/tag operations, delete routed to the OS
  trash, operation log.
- **Product**: i18n (ja/en), onboarding flow, Ed25519-signed license system
  with a 14-day trial, in-app pricing screen, opt-in telemetry (counters
  only — never file names or paths), auto-updater (GitHub Releases).
- **Release infra**: cross-platform signed-build CI (`.github/workflows/release.yml`),
  `latest.json` manifest generator, reference license server (`licenser/`),
  static marketing site (`landing/`), distribution/launch/commerce docs.

### Known limitations
- Cloud sync (S3 / Dropbox / iCloud) not yet implemented.
- PDF summarization requires bundling pdfium (currently text files only).
- Treemap uses slice-and-dice layout; squarified layout is planned.
- Rule editing UI still exposes raw template fields in places.
