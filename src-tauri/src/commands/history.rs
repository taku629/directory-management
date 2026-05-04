//! Operation history & undo.
//!
//! Every destructive command appends a row to `operations` with an
//! `inverse_json` payload. Undo reads that, replays it, and marks the
//! original row as undone. We only undo what we logged the inverse for —
//! `delete` is irreversible right now (Phase 2.x will move to system Trash).

use std::path::Path;

use rusqlite::params;
use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct OperationRow {
    pub id: i64,
    pub op_type: String,
    pub payload_json: String,
    pub inverse_json: Option<String>,
    pub triggered_by: String,
    pub created_at: i64,
    pub undone: bool,
}

#[tauri::command]
pub fn list_operations(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> AppResult<Vec<OperationRow>> {
    let conn = state.db.get()?;
    let limit = limit.unwrap_or(100).clamp(1, 1000);
    let mut stmt = conn.prepare(
        "SELECT id, op_type, payload_json, inverse_json, triggered_by, created_at, undone
         FROM operations ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map([limit], |r| {
            Ok(OperationRow {
                id: r.get(0)?,
                op_type: r.get(1)?,
                payload_json: r.get(2)?,
                inverse_json: r.get(3)?,
                triggered_by: r.get(4)?,
                created_at: r.get(5)?,
                undone: r.get::<_, i64>(6)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn undo_operation(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.get()?;
    let (op_type, inverse_json, undone): (String, Option<String>, i64) = conn
        .query_row(
            "SELECT op_type, inverse_json, undone FROM operations WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|_| AppError::NotFound(format!("operation {id}")))?;

    if undone != 0 {
        return Err(AppError::Invalid("already undone".into()));
    }
    let inverse = inverse_json
        .ok_or_else(|| AppError::Invalid(format!("{op_type} has no inverse logged")))?;
    let inv: serde_json::Value = serde_json::from_str(&inverse)?;

    match op_type.as_str() {
        "move" | "rename" => {
            let src = inv["src"].as_str().ok_or_else(|| AppError::Invalid("inverse.src".into()))?;
            let dst = inv["dst"].as_str().ok_or_else(|| AppError::Invalid("inverse.dst".into()))?;
            if let Some(parent) = Path::new(dst).parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(src, dst)?;
            let new_parent = Path::new(dst)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let new_name = Path::new(dst)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            conn.execute(
                "UPDATE files SET path = ?1, parent_path = ?2, name = ?3 WHERE path = ?4",
                params![dst, new_parent, new_name, src],
            )?;
        }
        "tag_add" => {
            let fid = inv["file_id"]
                .as_i64()
                .ok_or_else(|| AppError::Invalid("inverse.file_id".into()))?;
            let tid = inv["tag_id"]
                .as_i64()
                .ok_or_else(|| AppError::Invalid("inverse.tag_id".into()))?;
            conn.execute(
                "DELETE FROM file_tags WHERE file_id = ?1 AND tag_id = ?2",
                params![fid, tid],
            )?;
        }
        other => {
            return Err(AppError::Invalid(format!("undo not supported for: {other}")));
        }
    }

    conn.execute("UPDATE operations SET undone = 1 WHERE id = ?1", [id])?;
    Ok(())
}
