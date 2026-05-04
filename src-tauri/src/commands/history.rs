//! Operation history & undo — Phase 2.
//!
//! Every destructive command writes a row to `operations` with an inverse
//! payload. Undo replays the inverse and marks `undone = 1`. Phase 1 already
//! records ops; the Phase 2 patch fills in `inverse_json` and the executor.

use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct OperationRow {
    pub id: i64,
    pub op_type: String,
    pub payload_json: String,
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
        "SELECT id, op_type, payload_json, triggered_by, created_at, undone
         FROM operations ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map([limit], |r| {
            Ok(OperationRow {
                id: r.get(0)?,
                op_type: r.get(1)?,
                payload_json: r.get(2)?,
                triggered_by: r.get(3)?,
                created_at: r.get(4)?,
                undone: r.get::<_, i64>(5)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn undo_operation(_state: State<'_, AppState>, _id: i64) -> AppResult<()> {
    Err(AppError::NotImplemented(
        "undo executor lands in Phase 2 (paired with rule engine)",
    ))
}
