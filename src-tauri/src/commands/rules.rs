//! Rule CRUD — Phase 2.
//!
//! Rules persist into the `rules` table; `run_rule_now` and live triggering
//! via the watcher land in Phase 2 implementation.

use rusqlite::params;
use serde::Deserialize;
use tauri::State;

use crate::db::now_ts;
use crate::error::{AppError, AppResult};
use crate::rules::Rule;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RuleInput {
    pub name: String,
    pub enabled: bool,
    pub watched_path: String,
    pub conditions_json: String,
    pub actions_json: String,
    pub priority: i32,
}

#[tauri::command]
pub fn create_rule(state: State<'_, AppState>, rule: RuleInput) -> AppResult<i64> {
    let conn = state.db.get()?;
    conn.execute(
        "INSERT INTO rules (name, enabled, watched_path, conditions_json, actions_json,
                            priority, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            rule.name,
            rule.enabled as i64,
            rule.watched_path,
            rule.conditions_json,
            rule.actions_json,
            rule.priority,
            now_ts()
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn list_rules(state: State<'_, AppState>) -> AppResult<Vec<Rule>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, enabled, watched_path, conditions_json, actions_json,
                priority, last_run_at
         FROM rules ORDER BY priority DESC, id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            let conditions: Vec<crate::rules::Condition> =
                serde_json::from_str::<Vec<_>>(&r.get::<_, String>(4)?).unwrap_or_default();
            let actions: Vec<crate::rules::Action> =
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

#[tauri::command]
pub fn update_rule(
    state: State<'_, AppState>,
    id: i64,
    rule: RuleInput,
) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute(
        "UPDATE rules SET name = ?1, enabled = ?2, watched_path = ?3,
                          conditions_json = ?4, actions_json = ?5, priority = ?6
         WHERE id = ?7",
        params![
            rule.name,
            rule.enabled as i64,
            rule.watched_path,
            rule.conditions_json,
            rule.actions_json,
            rule.priority,
            id
        ],
    )?;
    Ok(())
}

#[tauri::command]
pub fn delete_rule(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.get()?;
    conn.execute("DELETE FROM rules WHERE id = ?1", [id])?;
    Ok(())
}

#[tauri::command]
pub fn run_rule_now(_state: State<'_, AppState>, _id: i64) -> AppResult<()> {
    Err(AppError::NotImplemented(
        "rule execution lands in Phase 2 (rules engine)",
    ))
}
