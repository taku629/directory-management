use tauri::State;

use crate::duplicates::{self, DuplicateGroup, SimilarGroup};
use crate::error::AppResult;
use crate::state::AppState;

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
    state: State<'_, AppState>,
    path: Option<String>,
    max_distance: Option<u32>,
) -> AppResult<Vec<SimilarGroup>> {
    duplicates::find_similar_images(
        &state.db,
        path.as_deref(),
        max_distance.unwrap_or(5),
    )
}
