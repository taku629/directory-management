//! Start / stop the live watcher over all enabled `watched_roots`.

use std::path::PathBuf;

use serde::Serialize;
use tauri::State;

use crate::error::AppResult;
use crate::fs::watcher;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct WatcherStatus {
    pub running: bool,
    pub roots: Vec<String>,
}

#[tauri::command]
pub fn start_watcher(state: State<'_, AppState>) -> AppResult<WatcherStatus> {
    let conn = state.db.get()?;
    let mut stmt =
        conn.prepare("SELECT path FROM watched_roots WHERE enabled = 1 ORDER BY id")?;
    let roots: Vec<PathBuf> = stmt
        .query_map([], |r| r.get::<_, String>(0).map(PathBuf::from))?
        .collect::<Result<_, _>>()?;

    let handle = watcher::start(roots.clone(), state.db.clone())?;
    *state.watcher.lock() = Some(handle);

    Ok(WatcherStatus {
        running: true,
        roots: roots.iter().map(|p| p.to_string_lossy().to_string()).collect(),
    })
}

#[tauri::command]
pub fn stop_watcher(state: State<'_, AppState>) -> AppResult<()> {
    *state.watcher.lock() = None;
    Ok(())
}

#[tauri::command]
pub fn watcher_status(state: State<'_, AppState>) -> AppResult<WatcherStatus> {
    let guard = state.watcher.lock();
    match guard.as_ref() {
        Some(h) => Ok(WatcherStatus {
            running: true,
            roots: h
                .roots
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
        }),
        None => Ok(WatcherStatus {
            running: false,
            roots: Vec::new(),
        }),
    }
}
