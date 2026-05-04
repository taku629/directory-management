use tauri::State;

use crate::duplicates::DuplicateGroup;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
pub fn find_duplicates(
    _state: State<'_, AppState>,
    _path: Option<String>,
) -> AppResult<Vec<DuplicateGroup>> {
    Err(AppError::NotImplemented(
        "duplicate detection lands in Phase 2 (sha256 indexer)",
    ))
}

#[tauri::command]
pub fn find_similar_images(
    _state: State<'_, AppState>,
    _path: Option<String>,
    _max_distance: Option<u32>,
) -> AppResult<Vec<DuplicateGroup>> {
    Err(AppError::NotImplemented(
        "perceptual-hash clustering lands in Phase 2",
    ))
}
