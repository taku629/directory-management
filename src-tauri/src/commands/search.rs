use tauri::State;

use crate::error::AppResult;
use crate::search::{indexer, SearchQuery, SearchResult};
use crate::state::AppState;

#[tauri::command]
pub fn search_files(state: State<'_, AppState>, query: SearchQuery) -> AppResult<SearchResult> {
    indexer::run(&state.db, &query)
}
