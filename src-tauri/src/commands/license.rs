//! License management commands.

use rusqlite::params;
use serde::Serialize;
use tauri::State;

use crate::db::now_ts;
use crate::error::{AppError, AppResult};
use crate::license::{
    entitlement as compute_entitlement, machine_id, verify::verify_license, Entitlement,
};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct LicenseInfo {
    pub tier: String,
    pub activated_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub machine_id: String,
    pub entitlement: Entitlement,
}

#[tauri::command]
pub fn get_license(state: State<'_, AppState>) -> AppResult<LicenseInfo> {
    let conn = state.db.get()?;
    let (tier, activated, expires): (String, Option<i64>, Option<i64>) = conn
        .query_row(
            "SELECT tier, activated_at, expires_at FROM license WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap_or_else(|_| ("free".to_string(), None, None));
    let mid = machine_id(&state)?;
    let ent = compute_entitlement(&state)?;
    Ok(LicenseInfo {
        tier,
        activated_at: activated,
        expires_at: expires,
        machine_id: mid,
        entitlement: ent,
    })
}

#[tauri::command]
pub fn activate_license(
    state: State<'_, AppState>,
    license_key: String,
) -> AppResult<LicenseInfo> {
    if license_key.trim().is_empty() {
        return Err(AppError::Invalid("empty license key".into()));
    }

    // Dev shortcut: SIFT-PRO-* / SIFT-TEAM-* with no signature still work
    // so we can exercise the gating end-to-end without running the licensing
    // service. Production keys are signed and routed through `verify_license`.
    let key = license_key.trim();
    let (tier, expires_at) = if key.contains('.') {
        let payload = verify_license(key)?;
        let our_machine = machine_id(&state)?;
        if payload.machine_id != our_machine {
            return Err(AppError::Invalid(
                "ライセンスがこのマシン用じゃない".into(),
            ));
        }
        if let Some(exp) = payload.expires_at {
            if exp < now_ts() {
                return Err(AppError::Invalid("ライセンス期限切れ".into()));
            }
        }
        (payload.tier, payload.expires_at)
    } else if key.starts_with("SIFT-PRO-") {
        ("pro".to_string(), None)
    } else if key.starts_with("SIFT-TEAM-") {
        ("team".to_string(), None)
    } else {
        return Err(AppError::Invalid(
            "ライセンスキーが不正。SIFT-PRO- か署名済みトークンを入れて。".into(),
        ));
    };

    let conn = state.db.get()?;
    conn.execute(
        "UPDATE license SET license_key = ?1, tier = ?2, activated_at = ?3, expires_at = ?4 WHERE id = 1",
        params![key, tier, now_ts(), expires_at],
    )?;
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

/// Returns the user's current entitlement — used by the UI to show
/// "Trial: 7 days left" badges and gate Pro features pre-emptively.
#[tauri::command]
pub fn get_entitlement(state: State<'_, AppState>) -> AppResult<Entitlement> {
    compute_entitlement(&state)
}
