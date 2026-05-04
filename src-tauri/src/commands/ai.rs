//! AI commands — Phase 3.
//!
//! All commands are wired so the frontend can call them today; they return
//! `LockedFeature` until the user activates a Pro license, and
//! `NotImplemented` once Pro is on but the backend isn't shipped yet.

use tauri::State;

use crate::ai::{AiSearchHit, AiTag};
use crate::error::{AppError, AppResult};
use crate::license::{require_tier, Tier};
use crate::state::AppState;

#[tauri::command]
pub fn ai_tag_file(state: State<'_, AppState>, _file_id: i64) -> AppResult<Vec<AiTag>> {
    require_tier(&state, Tier::Pro)?;
    Err(AppError::NotImplemented("ai_tag_file lands in Phase 3"))
}

#[tauri::command]
pub fn ai_classify_file(state: State<'_, AppState>, _file_id: i64) -> AppResult<String> {
    require_tier(&state, Tier::Pro)?;
    Err(AppError::NotImplemented("ai_classify_file lands in Phase 3"))
}

#[tauri::command]
pub fn ai_search(state: State<'_, AppState>, _query: String) -> AppResult<Vec<AiSearchHit>> {
    require_tier(&state, Tier::Pro)?;
    Err(AppError::NotImplemented("ai_search lands in Phase 3"))
}

#[tauri::command]
pub fn ai_summarize(state: State<'_, AppState>, _file_id: i64) -> AppResult<String> {
    require_tier(&state, Tier::Pro)?;
    Err(AppError::NotImplemented("ai_summarize lands in Phase 3"))
}

#[tauri::command]
pub fn ocr_file(state: State<'_, AppState>, _file_id: i64) -> AppResult<String> {
    require_tier(&state, Tier::Pro)?;
    Err(AppError::NotImplemented("ocr_file lands in Phase 3"))
}
