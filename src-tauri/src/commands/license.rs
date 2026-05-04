//! License management commands — Phase 4.

use rusqlite::params;
use serde::Serialize;
use tauri::State;

use crate::db::now_ts;
use crate::error::{AppError, AppResult};
use crate::license::{current_tier, Tier};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct LicenseInfo {
    pub tier: String,
    pub activated_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub machine_id: Option<String>,
}

#[tauri::command]
pub fn get_license(state: State<'_, AppState>) -> AppResult<LicenseInfo> {
    let conn = state.db.get()?;
    let row = conn.query_row(
        "SELECT tier, activated_at, expires_at, machine_id FROM license WHERE id = 1",
        [],
        |r| {
            Ok(LicenseInfo {
                tier: r.get(0)?,
                activated_at: r.get(1)?,
                expires_at: r.get(2)?,
                machine_id: r.get(3)?,
            })
        },
    )?;
    Ok(row)
}

#[tauri::command]
pub fn activate_license(
    state: State<'_, AppState>,
    license_key: String,
) -> AppResult<LicenseInfo> {
    if license_key.trim().is_empty() {
        return Err(AppError::Invalid("empty license key".into()));
    }
    // Phase 4: POST to licensing service, validate signed JWT, persist tier.
    // For Phase 1 plumbing, accept any key prefixed `SIFT-PRO-` so the gating
    // pathway can be exercised end-to-end during development.
    let tier = if license_key.starts_with("SIFT-PRO-") {
        Tier::Pro
    } else if license_key.starts_with("SIFT-TEAM-") {
        Tier::Team
    } else {
        return Err(AppError::Invalid(
            "license verification not yet implemented; use SIFT-PRO-xxxx for testing".into(),
        ));
    };

    let conn = state.db.get()?;
    conn.execute(
        "UPDATE license SET license_key = ?1, tier = ?2, activated_at = ?3 WHERE id = 1",
        params![license_key, tier.as_str(), now_ts()],
    )?;
    let _ = current_tier(&state)?;
    get_license(state)
}

#[tauri::command]
pub fn deactivate_license(state: State<'_, AppState>) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute(
        "UPDATE license SET license_key = NULL, tier = 'free',
                            activated_at = NULL, expires_at = NULL WHERE id = 1",
        [],
    )?;
    Ok(())
}
