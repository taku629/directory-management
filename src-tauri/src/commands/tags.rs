//! Tag CRUD and file<->tag association.

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::now_ts;
use crate::error::AppResult;
use crate::fs::FileEntry;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub parent_tag_id: Option<i64>,
    pub created_at: i64,
}

#[tauri::command]
pub fn create_tag(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
    parent_tag_id: Option<i64>,
) -> AppResult<i64> {
    let conn = state.db.get()?;
    conn.execute(
        "INSERT INTO tags (name, color, parent_tag_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, color, parent_tag_id, now_ts()],
    )?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> AppResult<Vec<Tag>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, color, parent_tag_id, created_at FROM tags ORDER BY name",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Tag {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                parent_tag_id: r.get(3)?,
                created_at: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn update_tag(
    state: State<'_, AppState>,
    id: i64,
    name: Option<String>,
    color: Option<String>,
    parent_tag_id: Option<i64>,
) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute(
        "UPDATE tags SET
            name = COALESCE(?1, name),
            color = COALESCE(?2, color),
            parent_tag_id = COALESCE(?3, parent_tag_id)
         WHERE id = ?4",
        params![name, color, parent_tag_id, id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn delete_tag(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute("DELETE FROM tags WHERE id = ?1", [id])?;
    Ok(())
}

#[tauri::command]
pub fn tag_file(
    state: State<'_, AppState>,
    file_id: i64,
    tag_id: i64,
    source: Option<String>,
) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute(
        "INSERT OR IGNORE INTO file_tags (file_id, tag_id, source, created_at) \
         VALUES (?1, ?2, ?3, ?4)",
        params![
            file_id,
            tag_id,
            source.unwrap_or_else(|| "user".into()),
            now_ts()
        ],
    )?;
    Ok(())
}

#[tauri::command]
pub fn untag_file(state: State<'_, AppState>, file_id: i64, tag_id: i64) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute(
        "DELETE FROM file_tags WHERE file_id = ?1 AND tag_id = ?2",
        params![file_id, tag_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn get_file_tags(state: State<'_, AppState>, file_id: i64) -> AppResult<Vec<Tag>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.parent_tag_id, t.created_at
         FROM tags t INNER JOIN file_tags ft ON ft.tag_id = t.id
         WHERE ft.file_id = ?1
         ORDER BY t.name",
    )?;
    let rows = stmt
        .query_map([file_id], |r| {
            Ok(Tag {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                parent_tag_id: r.get(3)?,
                created_at: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn list_files_by_tag(
    state: State<'_, AppState>,
    tag_id: i64,
) -> AppResult<Vec<FileEntry>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare(
        "SELECT f.id, f.path, f.name, f.parent_path, f.extension, f.size, f.mime_type,
                f.created_at, f.modified_at, f.is_directory, f.rating, f.color_label, f.note
         FROM files f INNER JOIN file_tags ft ON ft.file_id = f.id
         WHERE ft.tag_id = ?1 AND f.deleted = 0
         ORDER BY f.modified_at DESC",
    )?;
    let rows = stmt
        .query_map([tag_id], |r| {
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
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
