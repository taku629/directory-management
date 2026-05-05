//! License model & verification.
//!
//! ## Wire format
//!
//! ```text
//! SIFT-<TIER>-<RAW_KEY>.<BASE64URL_PAYLOAD>.<BASE64URL_SIGNATURE>
//! ```
//!
//! `RAW_KEY` is a short user-readable identifier the licensing service
//! issues (e.g. `9F2K-7Q4R`). `PAYLOAD` is JSON, signed by the licensing
//! service's Ed25519 private key. The client embeds the matching public
//! key at compile time and verifies offline.
//!
//! ## Trial
//!
//! On first launch we record `trial_started_at` and grant Pro for 14 days.
//! After the trial expires, gated features 403. The user can re-enter Pro
//! by activating a license key.
//!
//! ## Re-validation
//!
//! Phase 4.x: ping the licensing service every 7 days to confirm the key
//! hasn't been revoked. Offline grace period: 30 days.

pub mod verify;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Free,
    Pro,
    Team,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Free => "free",
            Tier::Pro => "pro",
            Tier::Team => "team",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "pro" => Tier::Pro,
            "team" => Tier::Team,
            _ => Tier::Free,
        }
    }
    pub fn rank(&self) -> u8 {
        match self {
            Tier::Free => 0,
            Tier::Pro => 1,
            Tier::Team => 2,
        }
    }
}

/// Effective entitlement: union of (paid license) and (trial).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlement {
    pub effective_tier: Tier,
    pub source: EntitlementSource,
    pub trial_days_left: Option<i64>,
    pub license_expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntitlementSource {
    Free,
    Trial,
    License,
}

/// Signed payload that the licensing service issues.
#[derive(Debug, Serialize, Deserialize)]
pub struct LicensePayload {
    pub tier: String,
    pub machine_id: String,
    pub email: Option<String>,
    pub issued_at: i64,
    pub expires_at: Option<i64>,
}

use tauri::State;

use crate::db::now_ts;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

const TRIAL_DAYS: i64 = 14;

/// Read or generate the per-machine ID, persisted in `settings`.
pub fn machine_id(state: &State<'_, AppState>) -> AppResult<String> {
    let conn = state.db.get()?;
    if let Ok(v) = conn.query_row(
        "SELECT value FROM settings WHERE key = 'machine_id'",
        [],
        |r| r.get::<_, String>(0),
    ) {
        return Ok(v);
    }
    // Prefer a stable hardware id; fall back to a random uuid v4.
    let id = machine_uid::get().unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('machine_id', ?1)",
        [&id],
    )?;
    Ok(id)
}

/// Days remaining in the trial. None if trial expired or not started.
pub fn trial_days_left(state: &State<'_, AppState>) -> AppResult<Option<i64>> {
    let conn = state.db.get()?;
    let started: Option<i64> = conn
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM settings WHERE key = 'trial_started_at'",
            [],
            |r| r.get(0),
        )
        .ok();
    let started = match started {
        Some(t) => t,
        None => {
            let now = now_ts();
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('trial_started_at', ?1)",
                [&now.to_string()],
            )?;
            now
        }
    };
    let elapsed = (now_ts() - started) / 86_400;
    let left = TRIAL_DAYS - elapsed;
    Ok(if left > 0 { Some(left) } else { None })
}

/// Compute the user's current entitlement: paid license trumps trial.
pub fn entitlement(state: &State<'_, AppState>) -> AppResult<Entitlement> {
    let conn = state.db.get()?;
    let row = conn
        .query_row(
            "SELECT tier, expires_at FROM license WHERE id = 1",
            [],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?)),
        )
        .unwrap_or_else(|_| ("free".to_string(), None));

    let licensed_tier = Tier::from_str(&row.0);
    let license_valid = licensed_tier != Tier::Free
        && row.1.map(|exp| exp > now_ts()).unwrap_or(true);

    if license_valid {
        return Ok(Entitlement {
            effective_tier: licensed_tier,
            source: EntitlementSource::License,
            trial_days_left: None,
            license_expires_at: row.1,
        });
    }

    let days = trial_days_left(state)?;
    Ok(if let Some(d) = days {
        Entitlement {
            effective_tier: Tier::Pro,
            source: EntitlementSource::Trial,
            trial_days_left: Some(d),
            license_expires_at: None,
        }
    } else {
        Entitlement {
            effective_tier: Tier::Free,
            source: EntitlementSource::Free,
            trial_days_left: None,
            license_expires_at: None,
        }
    })
}

pub fn require_tier(state: &State<'_, AppState>, min: Tier) -> AppResult<()> {
    let ent = entitlement(state)?;
    if ent.effective_tier.rank() >= min.rank() {
        Ok(())
    } else {
        Err(AppError::LockedFeature("this feature", min.as_str()))
    }
}

/// Backward-compat shim used by existing AI/cloud command modules.
pub fn current_tier(state: &State<'_, AppState>) -> AppResult<Tier> {
    Ok(entitlement(state)?.effective_tier)
}
