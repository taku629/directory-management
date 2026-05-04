//! File browsing, indexing, and metadata commands.

use std::path::Path;

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::now_ts;
use crate::error::{AppError, AppResult};
use crate::fs::{scanner, FileEntry};
use crate::state::AppState;

// --------- listing ---------

#[derive(Debug, Serialize)]
pub struct DirListing {
    pub path: String,
    pub entries: Vec<FileEntry>,
}

/// Live filesystem listing (does not require the path to be indexed).
/// Mirrors what Finder/Explorer would show, but enriched with whatever the DB
/// already knows (id/rating/color/note) so tags & metadata can be displayed.
#[tauri::command]
pub fn list_dir(state: State<'_, AppState>, path: String) -> AppResult<DirListing> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(path));
    }

    let conn = state.db.get()?;
    let mut entries: Vec<FileEntry> = Vec::new();

    for entry in std::fs::read_dir(p)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let path_str = entry.path().to_string_lossy().to_string();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let parent = p.to_string_lossy().to_string();
        let extension = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        let mime = mime_guess::from_path(entry.path())
            .first_raw()
            .map(|s| s.to_string());

        // Try to enrich with DB metadata if known.
        let enriched: Option<(i64, Option<i32>, Option<String>, Option<String>)> = conn
            .query_row(
                "SELECT id, rating, color_label, note FROM files WHERE path = ?1",
                [&path_str],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .ok();

        let (id, rating, color_label, note) = match enriched {
            Some((i, r, c, n)) => (Some(i), r, c, n),
            None => (None, None, None, None),
        };

        entries.push(FileEntry {
            id,
            path: path_str,
            name,
            parent_path: parent,
            extension,
            size: meta.len() as i64,
            mime_type: mime,
            created_at: meta
                .created()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            modified_at: meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            is_directory: meta.is_dir(),
            rating,
            color_label,
            note,
        });
    }

    // Directories first, then alphabetic.
    entries.sort_by(|a, b| {
        b.is_directory
            .cmp(&a.is_directory)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(DirListing { path, entries })
}

#[tauri::command]
pub fn get_file(state: State<'_, AppState>, id: i64) -> AppResult<FileEntry> {
    let conn = state.db.get()?;
    let entry = conn.query_row(
        "SELECT id, path, name, parent_path, extension, size, mime_type,
                created_at, modified_at, is_directory, rating, color_label, note
         FROM files WHERE id = ?1",
        [id],
        |r| {
            Ok(FileEntry {
                id: Some(r.get(0)?),
                path: r.get(1)?,
                name: r.get(2)?,
                parent_path: r.get(3)?,
                extension: r.get(4)?,
                size: r.get(5)?,
                mime_type: r.get(6)?,
                created_at: r.get(7)?,
                modified_at: r.get(8)?,
                is_directory: r.get::<_, i64>(9)? != 0,
                rating: r.get(10)?,
                color_label: r.get(11)?,
                note: r.get(12)?,
            })
        },
    )?;
    Ok(entry)
}

// --------- indexing ---------

#[derive(Debug, Serialize)]
pub struct IndexReport {
    pub indexed: usize,
    pub skipped: usize,
    pub errors: usize,
}

#[tauri::command]
pub fn index_directory(state: State<'_, AppState>, path: String) -> AppResult<IndexReport> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(path));
    }
    let report = scanner::scan_path(&state.db, p)?;
    log::info!(
        "index {}: indexed={}, skipped={}, errors={}",
        path,
        report.indexed,
        report.skipped,
        report.errors
    );
    Ok(IndexReport {
        indexed: report.indexed,
        skipped: report.skipped,
        errors: report.errors,
    })
}

// --------- metadata ---------

#[derive(Debug, Deserialize)]
pub struct FileMetadataPatch {
    pub rating: Option<i32>,
    pub color_label: Option<String>,
    pub note: Option<String>,
}

#[tauri::command]
pub fn update_file_metadata(
    state: State<'_, AppState>,
    id: i64,
    patch: FileMetadataPatch,
) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute(
        "UPDATE files SET
            rating = COALESCE(?1, rating),
            color_label = COALESCE(?2, color_label),
            note = COALESCE(?3, note)
         WHERE id = ?4",
        params![patch.rating, patch.color_label, patch.note, id],
    )?;
    // Refresh FTS row when note changes.
    if patch.note.is_some() {
        conn.execute(
            "INSERT OR REPLACE INTO files_fts(rowid, name, note) \
             SELECT id, name, COALESCE(note,'') FROM files WHERE id = ?1",
            [id],
        )?;
    }
    Ok(())
}

// --------- file ops ---------

#[tauri::command]
pub fn open_in_explorer(path: String) -> AppResult<()> {
    // Open the parent directory using the OS default file manager.
    let p = Path::new(&path);
    let target = if p.is_dir() { p } else { p.parent().unwrap_or(p) };

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(target).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(target).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(target).spawn()?;
    }
    Ok(())
}

#[tauri::command]
pub fn move_file(
    state: State<'_, AppState>,
    src: String,
    dst: String,
) -> AppResult<()> {
    std::fs::rename(&src, &dst)?;
    let conn = state.db.get()?;
    let new_parent = Path::new(&dst)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let new_name = Path::new(&dst)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    conn.execute(
        "UPDATE files SET path = ?1, parent_path = ?2, name = ?3 WHERE path = ?4",
        params![dst, new_parent, new_name, src],
    )?;
    record_op(&conn, "move", &serde_json::json!({"src": src, "dst": dst}))?;
    Ok(())
}

#[tauri::command]
pub fn rename_file(
    state: State<'_, AppState>,
    path: String,
    new_name: String,
) -> AppResult<String> {
    let src = Path::new(&path);
    let parent = src.parent().ok_or_else(|| AppError::Invalid("no parent".into()))?;
    let dst = parent.join(&new_name);
    std::fs::rename(src, &dst)?;
    let dst_str = dst.to_string_lossy().to_string();
    let conn = state.db.get()?;
    conn.execute(
        "UPDATE files SET path = ?1, name = ?2 WHERE path = ?3",
        params![dst_str, new_name, path],
    )?;
    record_op(
        &conn,
        "rename",
        &serde_json::json!({"src": path, "dst": dst_str}),
    )?;
    Ok(dst_str)
}

#[tauri::command]
pub fn delete_file(state: State<'_, AppState>, path: String) -> AppResult<()> {
    // Send to OS trash via the `trash` crate. If we ever want hard-delete
    // we can add a separate command — defaulting to recoverable is safer.
    trash::delete(&path).map_err(|e| AppError::Other(anyhow::anyhow!(e)))?;
    let conn = state.db.get()?;
    conn.execute("UPDATE files SET deleted = 1 WHERE path = ?1", [&path])?;

    // Inverse hint for undo: where the file used to live. The actual restore
    // walks the OS trash on supported platforms; on macOS we just point the
    // user at Finder's Trash.
    record_op(
        &conn,
        "delete",
        &serde_json::json!({ "path": path, "deleted_at": now_ts() }),
    )?;
    conn.execute(
        "UPDATE operations SET inverse_json = ?1
         WHERE id = (SELECT id FROM operations ORDER BY id DESC LIMIT 1)",
        [serde_json::json!({ "path": path }).to_string()],
    )?;
    Ok(())
}

fn record_op(
    conn: &rusqlite::Connection,
    op_type: &str,
    payload: &serde_json::Value,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO operations (op_type, payload_json, triggered_by, created_at) \
         VALUES (?1, ?2, 'user', ?3)",
        params![op_type, payload.to_string(), now_ts()],
    )?;
    Ok(())
}

// --------- watched roots ---------

#[derive(Debug, Serialize)]
pub struct WatchedRoot {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
    pub last_scan_at: Option<i64>,
    pub created_at: i64,
}

#[tauri::command]
pub fn add_watched_root(state: State<'_, AppState>, path: String) -> AppResult<i64> {
    let conn = state.db.get()?;
    conn.execute(
        "INSERT OR IGNORE INTO watched_roots (path, enabled, created_at) VALUES (?1, 1, ?2)",
        params![path, now_ts()],
    )?;
    let id: i64 = conn.query_row("SELECT id FROM watched_roots WHERE path = ?1", [&path], |r| {
        r.get(0)
    })?;
    Ok(id)
}

#[tauri::command]
pub fn list_watched_roots(state: State<'_, AppState>) -> AppResult<Vec<WatchedRoot>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, path, enabled, last_scan_at, created_at FROM watched_roots ORDER BY id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(WatchedRoot {
                id: r.get(0)?,
                path: r.get(1)?,
                enabled: r.get::<_, i64>(2)? != 0,
                last_scan_at: r.get(3)?,
                created_at: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn remove_watched_root(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute("DELETE FROM watched_roots WHERE id = ?1", [id])?;
    Ok(())
}

// --------- favorites ---------

#[derive(Debug, Serialize)]
pub struct Favorite {
    pub id: i64,
    pub path: String,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i64,
}

#[tauri::command]
pub fn add_favorite(
    state: State<'_, AppState>,
    path: String,
    name: Option<String>,
) -> AppResult<i64> {
    let conn = state.db.get()?;
    conn.execute(
        "INSERT INTO favorites (path, name, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![path, name, now_ts(), now_ts()],
    )?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn list_favorites(state: State<'_, AppState>) -> AppResult<Vec<Favorite>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, path, name, icon, sort_order FROM favorites ORDER BY sort_order, id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Favorite {
                id: r.get(0)?,
                path: r.get(1)?,
                name: r.get(2)?,
                icon: r.get(3)?,
                sort_order: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn remove_favorite(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute("DELETE FROM favorites WHERE id = ?1", [id])?;
    Ok(())
}

// --------- smart folders ---------

#[derive(Debug, Serialize)]
pub struct SmartFolder {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub query_json: String,
    pub sort_order: i64,
}

#[tauri::command]
pub fn create_smart_folder(
    state: State<'_, AppState>,
    name: String,
    query_json: String,
) -> AppResult<i64> {
    let conn = state.db.get()?;
    conn.execute(
        "INSERT INTO smart_folders (name, query_json, created_at) VALUES (?1, ?2, ?3)",
        params![name, query_json, now_ts()],
    )?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn list_smart_folders(state: State<'_, AppState>) -> AppResult<Vec<SmartFolder>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, icon, query_json, sort_order FROM smart_folders ORDER BY sort_order, id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SmartFolder {
                id: r.get(0)?,
                name: r.get(1)?,
                icon: r.get(2)?,
                query_json: r.get(3)?,
                sort_order: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn delete_smart_folder(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute("DELETE FROM smart_folders WHERE id = ?1", [id])?;
    Ok(())
}
