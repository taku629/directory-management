//! Duplicate file detection.
//!
//! Strategy: group by `size` first (cheap), then SHA-256 only the buckets
//! with ≥ 2 files. Hashes are persisted to `files.hash_sha256` so repeat
//! scans skip already-known files.
//!
//! For images we additionally compute a 64-bit perceptual hash (`phash`)
//! and cluster by Hamming distance.

pub mod phash;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use rusqlite::params;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::db::DbPool;
use crate::error::AppResult;

#[derive(Debug, Serialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: i64,
    pub files: Vec<DuplicateFile>,
}

#[derive(Debug, Serialize)]
pub struct DuplicateFile {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub modified_at: i64,
}

/// Stream-hash a file in 64KB chunks. Avoids loading huge files into memory.
pub fn hash_file(path: &Path) -> AppResult<String> {
    let mut f = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Find exact duplicates inside an optional path scope.
/// Persists computed hashes back into the `files` table.
pub fn find_exact(
    pool: &DbPool,
    scope: Option<&str>,
    min_size: i64,
) -> AppResult<Vec<DuplicateGroup>> {
    let conn = pool.get()?;

    // 1. find size buckets with ≥ 2 candidate files
    let (size_sql, size_params): (&str, Vec<rusqlite::types::Value>) = match scope {
        Some(prefix) => (
            "SELECT size FROM files
             WHERE deleted = 0 AND is_directory = 0 AND size >= ?1
               AND (parent_path = ?2 OR parent_path LIKE ?2 || '%')
             GROUP BY size HAVING COUNT(*) > 1",
            vec![min_size.into(), prefix.to_string().into()],
        ),
        None => (
            "SELECT size FROM files
             WHERE deleted = 0 AND is_directory = 0 AND size >= ?1
             GROUP BY size HAVING COUNT(*) > 1",
            vec![min_size.into()],
        ),
    };

    let candidate_sizes: Vec<i64> = {
        let mut stmt = conn.prepare(size_sql)?;
        let rows: Vec<i64> = stmt
            .query_map(rusqlite::params_from_iter(size_params.iter()), |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        rows
    };

    // 2. for each bucket, ensure each file has a hash
    for size in &candidate_sizes {
        let mut stmt = conn.prepare(
            "SELECT id, path, hash_sha256 FROM files
             WHERE size = ?1 AND deleted = 0 AND is_directory = 0",
        )?;
        let rows: Vec<(i64, String, Option<String>)> = stmt
            .query_map([size], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<Result<_, _>>()?;

        for (id, path, existing) in rows {
            if existing.is_some() {
                continue;
            }
            match hash_file(Path::new(&path)) {
                Ok(h) => {
                    conn.execute(
                        "UPDATE files SET hash_sha256 = ?1 WHERE id = ?2",
                        params![h, id],
                    )?;
                }
                Err(e) => log::warn!("hash {path}: {e}"),
            }
        }
    }

    // 3. group by hash
    let group_sql = match scope {
        Some(_) => {
            "SELECT hash_sha256, MIN(size)
             FROM files
             WHERE hash_sha256 IS NOT NULL AND deleted = 0 AND size >= ?1
               AND (parent_path = ?2 OR parent_path LIKE ?2 || '%')
             GROUP BY hash_sha256 HAVING COUNT(*) > 1
             ORDER BY MIN(size) DESC"
        }
        None => {
            "SELECT hash_sha256, MIN(size)
             FROM files
             WHERE hash_sha256 IS NOT NULL AND deleted = 0 AND size >= ?1
             GROUP BY hash_sha256 HAVING COUNT(*) > 1
             ORDER BY MIN(size) DESC"
        }
    };
    let group_params: Vec<rusqlite::types::Value> = match scope {
        Some(p) => vec![min_size.into(), p.to_string().into()],
        None => vec![min_size.into()],
    };

    let mut stmt = conn.prepare(group_sql)?;
    let groups: Vec<(String, i64)> = stmt
        .query_map(rusqlite::params_from_iter(group_params.iter()), |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<Result<_, _>>()?;

    let mut out = Vec::with_capacity(groups.len());
    for (hash, size) in groups {
        let mut stmt = conn.prepare(
            "SELECT id, path, name, modified_at FROM files
             WHERE hash_sha256 = ?1 AND deleted = 0
             ORDER BY modified_at",
        )?;
        let files = stmt
            .query_map([&hash], |r| {
                Ok(DuplicateFile {
                    id: r.get(0)?,
                    path: r.get(1)?,
                    name: r.get(2)?,
                    modified_at: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        out.push(DuplicateGroup { hash, size, files });
    }

    Ok(out)
}

#[derive(Debug, Serialize)]
pub struct SimilarGroup {
    pub representative: String,
    pub files: Vec<DuplicateFile>,
}

const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

/// Find visually similar images using dHash + Hamming distance.
/// `max_distance` defaults to 5 (≤ 8 % difference).
pub fn find_similar_images(
    pool: &DbPool,
    scope: Option<&str>,
    max_distance: u32,
) -> AppResult<Vec<SimilarGroup>> {
    let conn = pool.get()?;

    // 1. enumerate candidate image rows in scope
    let (sql, params): (&str, Vec<rusqlite::types::Value>) = match scope {
        Some(p) => (
            "SELECT id, path, name, modified_at, hash_phash, extension
             FROM files
             WHERE deleted = 0 AND is_directory = 0
               AND (parent_path = ?1 OR parent_path LIKE ?1 || '%')",
            vec![p.to_string().into()],
        ),
        None => (
            "SELECT id, path, name, modified_at, hash_phash, extension
             FROM files
             WHERE deleted = 0 AND is_directory = 0",
            vec![],
        ),
    };

    let candidates: Vec<(i64, String, String, i64, Option<String>, Option<String>)> = {
        let mut stmt = conn.prepare(sql)?;
        let rows: Vec<_> = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            })?
            .collect::<Result<_, _>>()?;
        rows
    };

    // 2. compute & persist phash for image files that don't have one
    let mut hashed: Vec<(i64, String, String, i64, u64)> = Vec::new();
    for (id, path, name, modified, existing, ext) in candidates {
        let is_image = ext
            .as_deref()
            .map(|e| IMAGE_EXTS.contains(&e))
            .unwrap_or(false);
        if !is_image {
            continue;
        }
        let h = match existing.as_deref().and_then(phash::hash_from_hex) {
            Some(h) => h,
            None => match phash::dhash(std::path::Path::new(&path)) {
                Ok(h) => {
                    let hex = phash::hex_of(h);
                    let _ = conn.execute(
                        "UPDATE files SET hash_phash = ?1 WHERE id = ?2",
                        params![hex, id],
                    );
                    h
                }
                Err(e) => {
                    log::warn!("phash {path}: {e}");
                    continue;
                }
            },
        };
        hashed.push((id, path, name, modified, h));
    }

    // 3. greedy cluster: each unclustered point becomes a center; pull in
    // anyone within `max_distance`. Good enough for browsable groups.
    let mut clusters: Vec<Vec<usize>> = Vec::new();
    let mut taken = vec![false; hashed.len()];
    for i in 0..hashed.len() {
        if taken[i] {
            continue;
        }
        let mut group = vec![i];
        taken[i] = true;
        for j in (i + 1)..hashed.len() {
            if taken[j] {
                continue;
            }
            if phash::hamming(hashed[i].4, hashed[j].4) <= max_distance {
                group.push(j);
                taken[j] = true;
            }
        }
        if group.len() >= 2 {
            clusters.push(group);
        }
    }

    Ok(clusters
        .into_iter()
        .map(|idxs| SimilarGroup {
            representative: hashed[idxs[0]].1.clone(),
            files: idxs
                .into_iter()
                .map(|i| {
                    let (id, path, name, modified, _) = &hashed[i];
                    DuplicateFile {
                        id: *id,
                        path: path.clone(),
                        name: name.clone(),
                        modified_at: *modified,
                    }
                })
                .collect(),
        })
        .collect())
}
