pub mod scanner;
pub mod thumbnail;
pub mod watcher;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: Option<i64>,
    pub path: String,
    pub name: String,
    pub parent_path: String,
    pub extension: Option<String>,
    pub size: i64,
    pub mime_type: Option<String>,
    pub created_at: i64,
    pub modified_at: i64,
    pub is_directory: bool,
    pub rating: Option<i32>,
    pub color_label: Option<String>,
    pub note: Option<String>,
}
