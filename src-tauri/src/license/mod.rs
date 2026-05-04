//! License tier gating — Phase 4.
//!
//! Phase 1 ships with everyone on `Tier::Free`. Activation flow:
//! 1. User pastes a key into Settings.
//! 2. We POST it to the licensing service with `machine_id`.
//! 3. Service returns a signed JWT containing tier + expiry.
//! 4. Token stored in `license` row; `current_tier` reads it.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

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

pub fn current_tier(state: &State<'_, AppState>) -> AppResult<Tier> {
    let conn = state.db.get()?;
    let tier: String = conn
        .query_row("SELECT tier FROM license WHERE id = 1", [], |r| r.get(0))
        .unwrap_or_else(|_| "free".to_string());
    Ok(Tier::from_str(&tier))
}

pub fn require_tier(state: &State<'_, AppState>, min: Tier) -> AppResult<()> {
    let cur = current_tier(state)?;
    if cur.rank() >= min.rank() {
        Ok(())
    } else {
        Err(AppError::LockedFeature(
            "this feature",
            min.as_str(),
        ))
    }
}
