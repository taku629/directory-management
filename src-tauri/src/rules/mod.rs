//! Rules engine — Phase 2.
//!
//! A rule binds a watched directory to (conditions, actions). When a file
//! lands in the directory and conditions match, the engine runs the actions
//! sequentially, recording each side-effect into `operations` so it can be
//! undone.
//!
//! ## Condition primitives (planned)
//! - `name_matches(regex)`         | `extension_in([".pdf", ...])`
//! - `mime_starts_with("image/")`  | `size_between(min, max)`
//! - `created_within(duration)`    | `path_is_in(prefix)`
//! - `has_tag(tag)`                | `ai_label_is(label, conf>=)`
//!
//! ## Action primitives (planned)
//! - `move_to(path_template)`      | `rename_to(template)`
//! - `add_tag(tag)`                | `set_color(label)`
//! - `set_rating(n)`               | `run_script(path)`
//! - `notify(message)`
//!
//! Templates support `{name}`, `{ext}`, `{date:Y/m/d}`, `{ai_category}` etc.

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
    NameMatches(String),
    ExtensionIn(Vec<String>),
    MimeStartsWith(String),
    SizeBetween { min: Option<i64>, max: Option<i64> },
    CreatedWithinDays(i64),
    PathIsIn(String),
    HasTag(i64),
    AiLabelIs { label: String, min_confidence: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Action {
    MoveTo(String),
    RenameTo(String),
    AddTag(i64),
    SetColor(String),
    SetRating(i32),
    RunScript(String),
    Notify(String),
}

// Engine implementation lands in Phase 2.
