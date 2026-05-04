//! Disk-usage tree-map data — Phase 2.

use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct DiskNode {
    pub path: String,
    pub name: String,
    pub size: i64,
    pub children: Vec<DiskNode>,
}

#[tauri::command]
pub fn compute_disk_usage(
    _state: State<'_, AppState>,
    _path: String,
    _max_depth: Option<u32>,
) -> AppResult<DiskNode> {
    Err(AppError::NotImplemented(
        "disk usage treemap lands in Phase 2",
    ))
}
