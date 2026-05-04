//! Rules engine.
//!
//! A rule binds a watched directory to (conditions, actions). When a file
//! lands in the directory and conditions match, actions run sequentially,
//! recording each side-effect into `operations` so it can be undone.

pub mod engine;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: i64,
    pub name: String,
    pub enabled: bool,
    pub watched_path: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub priority: i32,
    pub last_run_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Condition {
    /// Glob match against the file name (e.g. `IMG_*.jpg`)
    NameMatches(String),
    /// Lowercase extensions, no dot.
    ExtensionIn(Vec<String>),
    /// MIME prefix like `image/`.
    MimeStartsWith(String),
    SizeBetween {
        min: Option<i64>,
        max: Option<i64>,
    },
    /// File modified within the last N days.
    ModifiedWithinDays(i64),
    /// Parent path starts with this prefix.
    PathIsIn(String),
    HasTag(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Action {
    /// Template path. Variables: `{name}`, `{ext}`, `{stem}`,
    /// `{year}`, `{month}`, `{day}`.
    MoveTo(String),
    RenameTo(String),
    AddTag(i64),
    SetColor(String),
    SetRating(i32),
    Notify(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedOp {
    pub file_path: String,
    pub action: String,
    pub detail: String,
}
