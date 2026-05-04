pub mod indexer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Free text. Searched against name + note via FTS.
    pub text: Option<String>,
    /// Restrict to a parent path prefix.
    pub path_prefix: Option<String>,
    /// Comma-separated extensions like `["jpg","png"]`.
    pub extensions: Option<Vec<String>>,
    /// Tag ids.
    pub tag_ids: Option<Vec<i64>>,
    /// Min/max size bytes.
    pub size_min: Option<i64>,
    pub size_max: Option<i64>,
    /// Modified-after / before unix ts.
    pub modified_after: Option<i64>,
    pub modified_before: Option<i64>,
    /// Pagination
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub total: i64,
    pub items: Vec<crate::fs::FileEntry>,
}
