//! Directory walker that indexes files into the `files` table.
//!
//! The scanner is the workhorse of Phase 1: it traverses a watched root,
//! upserts metadata, and keeps the FTS index in sync. Heavy work
//! (hashing, perceptual hash, thumbnails) is deferred to background jobs
//! so the initial scan stays responsive.

use std::path::Path;
use std::time::SystemTime;

use rusqlite::params;
use walkdir::WalkDir;

use crate::db::{now_ts, DbPool};
use crate::error::AppResult;

pub struct ScanReport {
    pub indexed: usize,
    pub skipped: usize,
    pub errors: usize,
}

pub fn scan_path(pool: &DbPool, root: &Path) -> AppResult<ScanReport> {
    let mut indexed = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    {
        let mut upsert = tx.prepare(
            r#"
            INSERT INTO files (
                path, name, parent_path, extension, size, mime_type,
                created_at, modified_at, accessed_at, indexed_at, is_directory
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(path) DO UPDATE SET
                size = excluded.size,
                mime_type = excluded.mime_type,
                modified_at = excluded.modified_at,
                accessed_at = excluded.accessed_at,
                indexed_at = excluded.indexed_at,
                deleted = 0
            "#,
        )?;
        let mut fts = tx.prepare(
            "INSERT OR REPLACE INTO files_fts(rowid, name, note) \
             SELECT id, name, COALESCE(note,'') FROM files WHERE path = ?1",
        )?;

        for entry in WalkDir::new(root).follow_links(false) {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    log::warn!("walk error: {e}");
                    errors += 1;
                    continue;
                }
            };

            let path = entry.path();
            // Skip hidden dotfiles and common junk dirs to avoid noise.
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                skipped += 1;
                continue;
            }

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    log::warn!("metadata error for {}: {e}", path.display());
                    errors += 1;
                    continue;
                }
            };

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let parent = path
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());
            let mime = mime_guess::from_path(path).first_raw().map(|s| s.to_string());
            let size = meta.len() as i64;
            let created = system_time_to_ts(meta.created().ok());
            let modified = system_time_to_ts(meta.modified().ok());
            let accessed = system_time_to_ts(meta.accessed().ok());
            let is_dir = meta.is_dir() as i64;

            upsert.execute(params![
                path.to_string_lossy(),
                name,
                parent,
                ext,
                size,
                mime,
                created,
                modified,
                accessed,
                now_ts(),
                is_dir,
            ])?;
            fts.execute(params![path.to_string_lossy()])?;
            indexed += 1;
        }
    }
    tx.commit()?;

    Ok(ScanReport { indexed, skipped, errors })
}

fn system_time_to_ts(t: Option<SystemTime>) -> i64 {
    t.and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
