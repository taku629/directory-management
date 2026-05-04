//! Search execution. Phase 1 uses SQLite + FTS5; Phase 3 adds a vector
//! search side-table for AI semantic search.

use rusqlite::params_from_iter;
use rusqlite::types::Value;

use crate::db::DbPool;
use crate::error::AppResult;
use crate::fs::FileEntry;

use super::{SearchQuery, SearchResult};

pub fn run(pool: &DbPool, q: &SearchQuery) -> AppResult<SearchResult> {
    let mut where_parts: Vec<String> = vec!["files.deleted = 0".into()];
    let mut binds: Vec<Value> = Vec::new();

    if let Some(text) = q.text.as_deref().filter(|s| !s.is_empty()) {
        where_parts.push(
            "files.id IN (SELECT rowid FROM files_fts WHERE files_fts MATCH ?)".into(),
        );
        binds.push(Value::Text(fts_escape(text)));
    }
    if let Some(prefix) = q.path_prefix.as_deref().filter(|s| !s.is_empty()) {
        where_parts.push("files.parent_path LIKE ? || '%'".into());
        binds.push(Value::Text(prefix.to_string()));
    }
    if let Some(exts) = q.extensions.as_ref().filter(|v| !v.is_empty()) {
        let placeholders = vec!["?"; exts.len()].join(",");
        where_parts.push(format!("files.extension IN ({placeholders})"));
        for e in exts {
            binds.push(Value::Text(e.to_lowercase()));
        }
    }
    if let Some(tag_ids) = q.tag_ids.as_ref().filter(|v| !v.is_empty()) {
        let placeholders = vec!["?"; tag_ids.len()].join(",");
        where_parts.push(format!(
            "files.id IN (SELECT file_id FROM file_tags WHERE tag_id IN ({placeholders}))"
        ));
        for t in tag_ids {
            binds.push(Value::Integer(*t));
        }
    }
    if let Some(min) = q.size_min {
        where_parts.push("files.size >= ?".into());
        binds.push(Value::Integer(min));
    }
    if let Some(max) = q.size_max {
        where_parts.push("files.size <= ?".into());
        binds.push(Value::Integer(max));
    }
    if let Some(ts) = q.modified_after {
        where_parts.push("files.modified_at >= ?".into());
        binds.push(Value::Integer(ts));
    }
    if let Some(ts) = q.modified_before {
        where_parts.push("files.modified_at <= ?".into());
        binds.push(Value::Integer(ts));
    }

    let where_sql = where_parts.join(" AND ");
    let limit = q.limit.unwrap_or(200).clamp(1, 1000);
    let offset = q.offset.unwrap_or(0).max(0);

    let conn = pool.get()?;

    let total: i64 = {
        let sql = format!("SELECT COUNT(*) FROM files WHERE {where_sql}");
        let mut stmt = conn.prepare(&sql)?;
        stmt.query_row(params_from_iter(binds.iter()), |r| r.get(0))?
    };

    let sql = format!(
        "SELECT id, path, name, parent_path, extension, size, mime_type,
                created_at, modified_at, is_directory, rating, color_label, note
         FROM files
         WHERE {where_sql}
         ORDER BY modified_at DESC
         LIMIT ? OFFSET ?"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut row_binds = binds.clone();
    row_binds.push(Value::Integer(limit));
    row_binds.push(Value::Integer(offset));

    let rows = stmt
        .query_map(params_from_iter(row_binds.iter()), |row| {
            Ok(FileEntry {
                id: Some(row.get(0)?),
                path: row.get(1)?,
                name: row.get(2)?,
                parent_path: row.get(3)?,
                extension: row.get(4)?,
                size: row.get(5)?,
                mime_type: row.get(6)?,
                created_at: row.get(7)?,
                modified_at: row.get(8)?,
                is_directory: row.get::<_, i64>(9)? != 0,
                rating: row.get(10)?,
                color_label: row.get(11)?,
                note: row.get(12)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SearchResult { total, items: rows })
}

/// Escape an FTS5 query string. Splits on whitespace and quotes each token
/// to make the query safe against operators while still acting as AND.
fn fts_escape(input: &str) -> String {
    input
        .split_whitespace()
        .map(|t| {
            let cleaned = t.replace('"', "\"\"");
            format!("\"{cleaned}\"*")
        })
        .collect::<Vec<_>>()
        .join(" ")
}
