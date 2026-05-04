use tauri::State;

use crate::duplicates::{self, DuplicateGroup};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Find exact duplicates within an optional path prefix.
/// `min_size` defaults to 4KB to skip noise.
#[tauri::command]
pub fn find_duplicates(
    state: State<'_, AppState>,
    path: Option<String>,
    min_size: Option<i64>,
) -> AppResult<Vec<DuplicateGroup>> {
    duplicates::find_exact(&state.db, path.as_deref(), min_size.unwrap_or(4096))
}

#[tauri::command]
pub fn find_similar_images(
    _state: State<'_, AppState>,
    _path: Option<String>,
    _max_distance: Option<u32>,
) -> AppResult<Vec<DuplicateGroup>> {
    Err(AppError::NotImplemented(
        "perceptual-hash clustering: 後で image_hasher 入れてやる",
    ))
}
