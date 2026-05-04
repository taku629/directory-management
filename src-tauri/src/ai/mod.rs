//! AI features (Phase 3, Pro tier).
//!
//! Backed by the Claude API. The API key is stored per-machine in the
//! `settings` table under `claude_api_key`. Without one, every AI command
//! returns an `Invalid("missing claude_api_key")` error so the UI can
//! prompt to add one.

pub mod claude;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTag {
    pub label: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize)]
pub struct AiSearchHit {
    pub file_id: i64,
    pub path: String,
    pub name: String,
    pub score: f32,
    pub reason: String,
}
