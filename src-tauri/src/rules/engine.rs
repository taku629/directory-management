//! Rule evaluation and execution.
//!
//! Conditions read either from a `FileEntry` row (when the file is indexed)
//! or directly from `std::fs` (when the watcher fires before indexing).
//! Actions mutate the filesystem and write op-log rows for future undo.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Datelike, TimeZone, Utc};
use globset::Glob;
use rusqlite::params;

use crate::db::{now_ts, DbPool};
use crate::error::{AppError, AppResult};

use super::{Action, Condition, PlannedOp, Rule};

/// Snapshot of a file used for condition evaluation. Built either from the
/// DB row or live from std::fs.
pub struct Candidate {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: i64,
    pub mime: Option<String>,
    pub modified_at: i64,
    pub file_id: Option<i64>,
}

impl Candidate {
    pub fn from_path(p: &Path) -> AppResult<Self> {
        let meta = std::fs::metadata(p)?;
        Ok(Self {
            path: p.to_path_buf(),
            name: p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            extension: p
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase()),
            size: meta.len() as i64,
            mime: mime_guess::from_path(p).first_raw().map(|s| s.to_string()),
            modified_at: meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            file_id: None,
        })
    }
}

/// Evaluate all conditions (AND). Tag conditions need DB access.
pub fn matches(pool: &DbPool, c: &Candidate, rule: &Rule) -> AppResult<bool> {
    for cond in &rule.conditions {
        if !eval(pool, c, cond)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn eval(pool: &DbPool, c: &Candidate, cond: &Condition) -> AppResult<bool> {
    Ok(match cond {
        Condition::NameMatches(pat) => Glob::new(pat)
            .map(|g| g.compile_matcher().is_match(&c.name))
            .unwrap_or(false),
        Condition::ExtensionIn(exts) => {
            c.extension.as_ref().map_or(false, |e| exts.iter().any(|x| x.eq_ignore_ascii_case(e)))
        }
        Condition::MimeStartsWith(p) => c.mime.as_ref().map_or(false, |m| m.starts_with(p)),
        Condition::SizeBetween { min, max } => {
            min.map_or(true, |m| c.size >= m) && max.map_or(true, |m| c.size <= m)
        }
        Condition::ModifiedWithinDays(d) => {
            let cutoff = now_ts() - d * 86_400;
            c.modified_at >= cutoff
        }
        Condition::PathIsIn(prefix) => c
            .path
            .parent()
            .map_or(false, |p| p.to_string_lossy().starts_with(prefix)),
        Condition::HasTag(tag_id) => match c.file_id {
            None => false,
            Some(fid) => {
                let conn = pool.get()?;
                conn.query_row(
                    "SELECT 1 FROM file_tags WHERE file_id = ?1 AND tag_id = ?2",
                    params![fid, tag_id],
                    |_| Ok(true),
                )
                .unwrap_or(false)
            }
        },
    })
}

/// Render the path/name template.
/// Supported tokens: {name} {stem} {ext} {year} {month} {day}
fn render_template(tmpl: &str, c: &Candidate) -> String {
    let stem = Path::new(&c.name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| c.name.clone());
    let ext = c.extension.clone().unwrap_or_default();

    let modified = Utc.timestamp_opt(c.modified_at, 0).single();
    let (y, m, d) = match modified {
        Some(t) => (
            format!("{:04}", t.year()),
            format!("{:02}", t.month()),
            format!("{:02}", t.day()),
        ),
        None => ("0000".into(), "00".into(), "00".into()),
    };

    tmpl.replace("{name}", &c.name)
        .replace("{stem}", &stem)
        .replace("{ext}", &ext)
        .replace("{year}", &y)
        .replace("{month}", &m)
        .replace("{day}", &d)
}

/// Plan (don't execute) what would happen if `rule` ran against `c`.
/// Used for dry-run and the "preview" pane.
pub fn plan(c: &Candidate, rule: &Rule) -> Vec<PlannedOp> {
    rule.actions
        .iter()
        .map(|a| match a {
            Action::MoveTo(t) => PlannedOp {
                file_path: c.path.to_string_lossy().to_string(),
                action: "move".into(),
                detail: render_template(t, c),
            },
            Action::RenameTo(t) => PlannedOp {
                file_path: c.path.to_string_lossy().to_string(),
                action: "rename".into(),
                detail: render_template(t, c),
            },
            Action::AddTag(id) => PlannedOp {
                file_path: c.path.to_string_lossy().to_string(),
                action: "add_tag".into(),
                detail: format!("tag #{id}"),
            },
            Action::SetColor(label) => PlannedOp {
                file_path: c.path.to_string_lossy().to_string(),
                action: "set_color".into(),
                detail: label.clone(),
            },
            Action::SetRating(n) => PlannedOp {
                file_path: c.path.to_string_lossy().to_string(),
                action: "set_rating".into(),
                detail: n.to_string(),
            },
            Action::Notify(msg) => PlannedOp {
                file_path: c.path.to_string_lossy().to_string(),
                action: "notify".into(),
                detail: msg.clone(),
            },
        })
        .collect()
}

/// Run all actions in order. Each successful op is logged with an inverse
/// payload so undo can replay it.
pub fn execute(
    pool: &DbPool,
    c: &Candidate,
    rule: &Rule,
    triggered_by: &str,
) -> AppResult<Vec<PlannedOp>> {
    let mut applied = Vec::new();
    let mut current_path = c.path.clone();

    for action in &rule.actions {
        match action {
            Action::MoveTo(tmpl) => {
                let dst = render_template(tmpl, c);
                let dst_path = resolve_destination(&dst, &current_path);
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::rename(&current_path, &dst_path)?;
                log_op(
                    pool,
                    "move",
                    &serde_json::json!({"src": current_path, "dst": dst_path}),
                    Some(&serde_json::json!({"src": dst_path, "dst": current_path})),
                    triggered_by,
                )?;
                update_db_path(pool, &current_path, &dst_path)?;
                current_path = dst_path;
                applied.push(PlannedOp {
                    file_path: current_path.to_string_lossy().to_string(),
                    action: "move".into(),
                    detail: tmpl.clone(),
                });
            }
            Action::RenameTo(tmpl) => {
                let new_name = render_template(tmpl, c);
                let parent = current_path.parent().unwrap_or(Path::new(""));
                let dst_path = parent.join(&new_name);
                std::fs::rename(&current_path, &dst_path)?;
                log_op(
                    pool,
                    "rename",
                    &serde_json::json!({"src": current_path, "dst": dst_path}),
                    Some(&serde_json::json!({"src": dst_path, "dst": current_path})),
                    triggered_by,
                )?;
                update_db_path(pool, &current_path, &dst_path)?;
                current_path = dst_path;
                applied.push(PlannedOp {
                    file_path: current_path.to_string_lossy().to_string(),
                    action: "rename".into(),
                    detail: new_name,
                });
            }
            Action::AddTag(tag_id) => {
                let conn = pool.get()?;
                if let Some(fid) = lookup_file_id(&conn, &current_path)? {
                    conn.execute(
                        "INSERT OR IGNORE INTO file_tags (file_id, tag_id, source, created_at) \
                         VALUES (?1, ?2, 'rule', ?3)",
                        params![fid, tag_id, now_ts()],
                    )?;
                    log_op(
                        pool,
                        "tag_add",
                        &serde_json::json!({"file_id": fid, "tag_id": tag_id}),
                        Some(&serde_json::json!({"file_id": fid, "tag_id": tag_id})),
                        triggered_by,
                    )?;
                    applied.push(PlannedOp {
                        file_path: current_path.to_string_lossy().to_string(),
                        action: "add_tag".into(),
                        detail: tag_id.to_string(),
                    });
                }
            }
            Action::SetColor(label) => {
                set_metadata(pool, &current_path, |conn, fid| {
                    conn.execute(
                        "UPDATE files SET color_label = ?1 WHERE id = ?2",
                        params![label, fid],
                    )?;
                    Ok(())
                })?;
                applied.push(PlannedOp {
                    file_path: current_path.to_string_lossy().to_string(),
                    action: "set_color".into(),
                    detail: label.clone(),
                });
            }
            Action::SetRating(n) => {
                let v = *n;
                set_metadata(pool, &current_path, |conn, fid| {
                    conn.execute(
                        "UPDATE files SET rating = ?1 WHERE id = ?2",
                        params![v, fid],
                    )?;
                    Ok(())
                })?;
                applied.push(PlannedOp {
                    file_path: current_path.to_string_lossy().to_string(),
                    action: "set_rating".into(),
                    detail: n.to_string(),
                });
            }
            Action::Notify(msg) => {
                log::info!("[rule:{}] notify: {}", rule.name, msg);
                applied.push(PlannedOp {
                    file_path: current_path.to_string_lossy().to_string(),
                    action: "notify".into(),
                    detail: msg.clone(),
                });
            }
        }
    }

    Ok(applied)
}

fn resolve_destination(rendered: &str, src: &Path) -> PathBuf {
    let p = Path::new(rendered);
    // If rendered template is a directory (no extension or trailing slash),
    // append the source filename.
    let looks_like_dir = rendered.ends_with('/') || rendered.ends_with('\\') || p.extension().is_none();
    if looks_like_dir {
        if let Some(name) = src.file_name() {
            return p.join(name);
        }
    }
    p.to_path_buf()
}

fn lookup_file_id(conn: &rusqlite::Connection, path: &Path) -> AppResult<Option<i64>> {
    Ok(conn
        .query_row(
            "SELECT id FROM files WHERE path = ?1 AND deleted = 0",
            [path.to_string_lossy()],
            |r| r.get(0),
        )
        .ok())
}

fn set_metadata<F>(pool: &DbPool, path: &Path, f: F) -> AppResult<()>
where
    F: FnOnce(&rusqlite::Connection, i64) -> AppResult<()>,
{
    let conn = pool.get()?;
    if let Some(fid) = lookup_file_id(&conn, path)? {
        f(&conn, fid)?;
    }
    Ok(())
}

fn update_db_path(pool: &DbPool, src: &Path, dst: &Path) -> AppResult<()> {
    let conn = pool.get()?;
    let new_parent = dst
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let new_name = dst
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    conn.execute(
        "UPDATE files SET path = ?1, parent_path = ?2, name = ?3 WHERE path = ?4",
        params![
            dst.to_string_lossy(),
            new_parent,
            new_name,
            src.to_string_lossy()
        ],
    )?;
    Ok(())
}

fn log_op(
    pool: &DbPool,
    op_type: &str,
    payload: &serde_json::Value,
    inverse: Option<&serde_json::Value>,
    triggered_by: &str,
) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO operations (op_type, payload_json, inverse_json, triggered_by, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            op_type,
            payload.to_string(),
            inverse.map(|v| v.to_string()),
            triggered_by,
            now_ts()
        ],
    )?;
    Ok(())
}

/// Load all enabled rules whose `watched_path` is a prefix of `under`.
pub fn rules_for_path(pool: &DbPool, under: &Path) -> AppResult<Vec<Rule>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, enabled, watched_path, conditions_json, actions_json,
                priority, last_run_at
         FROM rules
         WHERE enabled = 1 AND ?1 LIKE watched_path || '%'
         ORDER BY priority DESC, id",
    )?;
    let rows = stmt
        .query_map([under.to_string_lossy()], |r| {
            let conditions: Vec<Condition> =
                serde_json::from_str::<Vec<_>>(&r.get::<_, String>(4)?).unwrap_or_default();
            let actions: Vec<Action> =
                serde_json::from_str::<Vec<_>>(&r.get::<_, String>(5)?).unwrap_or_default();
            Ok(Rule {
                id: r.get(0)?,
                name: r.get(1)?,
                enabled: r.get::<_, i64>(2)? != 0,
                watched_path: r.get(3)?,
                conditions,
                actions,
                priority: r.get(6)?,
                last_run_at: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Mark a rule's last_run_at to now.
pub fn touch_last_run(pool: &DbPool, rule_id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE rules SET last_run_at = ?1 WHERE id = ?2",
        params![now_ts(), rule_id],
    )?;
    Ok(())
}

/// Convenience: load a single rule by id.
pub fn load_rule(pool: &DbPool, id: i64) -> AppResult<Rule> {
    let conn = pool.get()?;
    let r = conn
        .query_row(
            "SELECT id, name, enabled, watched_path, conditions_json, actions_json,
                    priority, last_run_at
             FROM rules WHERE id = ?1",
            [id],
            |r| {
                let conditions: Vec<Condition> =
                    serde_json::from_str::<Vec<_>>(&r.get::<_, String>(4)?).unwrap_or_default();
                let actions: Vec<Action> =
                    serde_json::from_str::<Vec<_>>(&r.get::<_, String>(5)?).unwrap_or_default();
                Ok(Rule {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    enabled: r.get::<_, i64>(2)? != 0,
                    watched_path: r.get(3)?,
                    conditions,
                    actions,
                    priority: r.get(6)?,
                    last_run_at: r.get(7)?,
                })
            },
        )
        .map_err(|_| AppError::NotFound(format!("rule {id}")))?;
    Ok(r)
}
