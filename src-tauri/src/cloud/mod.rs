//! Cloud sync — Phase 4.
//!
//! Targeted providers: Amazon S3, Backblaze B2, Dropbox, Google Drive,
//! iCloud Drive, OneDrive. Sync is one-way (local -> cloud snapshot) on
//! launch, with manual two-way merge for tag changes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudProvider {
    pub id: String,
    pub name: String,
    pub auth_kind: AuthKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthKind {
    OAuth,
    AccessKey,
    LocalPath, // iCloud Drive on macOS, OneDrive on Windows
}
