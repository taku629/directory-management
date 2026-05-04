//! Cloud sync commands — Phase 4.

use serde::Serialize;
use tauri::State;

use crate::cloud::{AuthKind, CloudProvider};
use crate::error::{AppError, AppResult};
use crate::license::{require_tier, Tier};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SyncReport {
    pub provider: String,
    pub uploaded: u64,
    pub bytes: u64,
}

#[tauri::command]
pub fn list_cloud_providers() -> AppResult<Vec<CloudProvider>> {
    Ok(vec![
        CloudProvider { id: "s3".into(), name: "Amazon S3".into(), auth_kind: AuthKind::AccessKey },
        CloudProvider { id: "b2".into(), name: "Backblaze B2".into(), auth_kind: AuthKind::AccessKey },
        CloudProvider { id: "dropbox".into(), name: "Dropbox".into(), auth_kind: AuthKind::OAuth },
        CloudProvider { id: "gdrive".into(), name: "Google Drive".into(), auth_kind: AuthKind::OAuth },
        CloudProvider { id: "icloud".into(), name: "iCloud Drive".into(), auth_kind: AuthKind::LocalPath },
        CloudProvider { id: "onedrive".into(), name: "OneDrive".into(), auth_kind: AuthKind::LocalPath },
    ])
}

#[tauri::command]
pub fn configure_cloud(
    state: State<'_, AppState>,
    _provider_id: String,
    _config_json: String,
) -> AppResult<()> {
    require_tier(&state, Tier::Pro)?;
    Err(AppError::NotImplemented("cloud configuration lands in Phase 4"))
}

#[tauri::command]
pub fn sync_now(state: State<'_, AppState>, _provider_id: String) -> AppResult<SyncReport> {
    require_tier(&state, Tier::Pro)?;
    Err(AppError::NotImplemented("cloud sync lands in Phase 4"))
}
