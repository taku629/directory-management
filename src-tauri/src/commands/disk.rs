//! Disk-usage tree builder.
//!
//! Walks the filesystem (live, not the index) and aggregates sizes into a
//! tree clipped at `max_depth`. Designed to feed the treemap.

use std::path::Path;

use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct DiskNode {
    pub path: String,
    pub name: String,
    pub size: i64,
    pub is_directory: bool,
    pub children: Vec<DiskNode>,
}

#[tauri::command]
pub fn compute_disk_usage(
    _state: State<'_, AppState>,
    path: String,
    max_depth: Option<u32>,
) -> AppResult<DiskNode> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(path));
    }
    let depth = max_depth.unwrap_or(3).clamp(1, 6);
    let node = walk(p, depth)?;
    Ok(node)
}

fn walk(p: &Path, depth_left: u32) -> AppResult<DiskNode> {
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| p.to_string_lossy().to_string());

    let meta = std::fs::metadata(p)?;
    if !meta.is_dir() {
        return Ok(DiskNode {
            path: p.to_string_lossy().to_string(),
            name,
            size: meta.len() as i64,
            is_directory: false,
            children: Vec::new(),
        });
    }

    let mut children: Vec<DiskNode> = Vec::new();
    let mut total: i64 = 0;

    if depth_left == 0 {
        // Just sum bytes without descending into another node level.
        for entry in walkdir::WalkDir::new(p) {
            match entry {
                Ok(e) => {
                    if let Ok(m) = e.metadata() {
                        if m.is_file() {
                            total += m.len() as i64;
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    } else {
        let read = match std::fs::read_dir(p) {
            Ok(r) => r,
            Err(_) => {
                return Ok(DiskNode {
                    path: p.to_string_lossy().to_string(),
                    name,
                    size: 0,
                    is_directory: true,
                    children: Vec::new(),
                });
            }
        };

        for entry in read.flatten() {
            let cp = entry.path();
            // skip symlinks to avoid loops
            if let Ok(ft) = entry.file_type() {
                if ft.is_symlink() {
                    continue;
                }
            }
            match walk(&cp, depth_left - 1) {
                Ok(child) => {
                    total += child.size;
                    children.push(child);
                }
                Err(e) => log::warn!("walk {}: {e}", cp.display()),
            }
        }
        children.sort_by_key(|c| std::cmp::Reverse(c.size));
        // Trim long tails to keep payload light; "Other" bucket sums the rest.
        if children.len() > 32 {
            let extras = children.split_off(32);
            let other_size: i64 = extras.iter().map(|c| c.size).sum();
            if other_size > 0 {
                children.push(DiskNode {
                    path: format!("{}/(other)", p.to_string_lossy()),
                    name: format!("(他 {} 件)", extras.len()),
                    size: other_size,
                    is_directory: false,
                    children: Vec::new(),
                });
            }
        }
    }

    Ok(DiskNode {
        path: p.to_string_lossy().to_string(),
        name,
        size: total,
        is_directory: true,
        children,
    })
}
