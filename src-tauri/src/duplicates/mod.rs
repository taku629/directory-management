//! Duplicate & near-duplicate detection — Phase 2.
//!
//! ## Strategy
//! - **Exact duplicates**: hash files in size buckets with SHA-256, group by
//!   hash. Background job populates `files.hash_sha256`.
//! - **Similar images**: 64-bit perceptual hash via `image_hasher`, then
//!   Hamming-distance clustering. Stored in `files.hash_phash`.
//! - **Similar documents** (Phase 3): MinHash over shingled text.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DuplicateGroup {
    pub key: String,
    pub size: i64,
    pub file_ids: Vec<i64>,
}
