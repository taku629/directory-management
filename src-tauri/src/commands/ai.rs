//! AI commands — Phase 3 (Pro tier).
//!
//! All commands route through the Claude API client and cache results in
//! the `ai_tags` / `ai_summaries` / `ocr_text` tables to avoid double
//! billing.

use std::path::Path;

use rusqlite::params;
use serde_json::{json, Value};
use tauri::State;

use crate::ai::claude::Client;
use crate::ai::{AiSearchHit, AiTag};
use crate::db::now_ts;
use crate::error::{AppError, AppResult};
use crate::license::{require_tier, Tier};
use crate::search::{indexer, SearchQuery};
use crate::state::AppState;

const VISION_MODEL: &str = "claude-sonnet-4-6";

fn client(state: &State<'_, AppState>) -> AppResult<Client> {
    require_tier(state, Tier::Pro)?;
    let conn = state.db.get()?;
    let key: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'claude_api_key'",
            [],
            |r| r.get(0),
        )
        .map_err(|_| AppError::Invalid("missing claude_api_key in settings".into()))?;
    if key.trim().is_empty() {
        return Err(AppError::Invalid("claude_api_key is empty".into()));
    }
    Ok(Client::new(key, Some(VISION_MODEL.to_string())))
}

fn lookup_path(state: &State<'_, AppState>, file_id: i64) -> AppResult<String> {
    let conn = state.db.get()?;
    conn.query_row("SELECT path FROM files WHERE id = ?1", [file_id], |r| {
        r.get::<_, String>(0)
    })
    .map_err(|_| AppError::NotFound(format!("file {file_id}")))
}

#[tauri::command]
pub async fn ai_tag_file(
    state: State<'_, AppState>,
    file_id: i64,
) -> AppResult<Vec<AiTag>> {
    let cli = client(&state)?;
    let path = lookup_path(&state, file_id)?;
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(path));
    }

    // Ask Claude for an array of tags via tool-use → strict schema.
    let schema = json!({
        "type": "object",
        "properties": {
            "tags": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string" },
                        "confidence": { "type": "number" }
                    },
                    "required": ["label", "confidence"]
                }
            }
        },
        "required": ["tags"]
    });
    let prompt = "この画像の内容を表すタグを 3〜8 個、英語の小文字 1〜2 単語で。\
                  confidence は 0.0〜1.0。例: dog, beach, sunset, screenshot, code, receipt";

    let result = cli
        .tool_call(None, prompt, "tag_image", schema)
        .await
        .or_else(|_| {
            // Fallback to vision if the model didn't tool-call.
            futures_block_workaround()
        })?;

    let tags: Vec<AiTag> = result["tags"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    Some(AiTag {
                        label: v["label"].as_str()?.to_lowercase(),
                        confidence: v["confidence"].as_f64()? as f32,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let conn = state.db.get()?;
    conn.execute("DELETE FROM ai_tags WHERE file_id = ?1", [file_id])?;
    for t in &tags {
        conn.execute(
            "INSERT INTO ai_tags (file_id, label, confidence, model, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![file_id, t.label, t.confidence, VISION_MODEL, now_ts()],
        )?;
    }

    // Also ensure these labels exist as user-facing tags so they show in
    // the sidebar without manual setup.
    for t in &tags {
        if t.confidence < 0.6 {
            continue;
        }
        let tag_id: i64 = match conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            [&t.label],
            |r| r.get(0),
        ) {
            Ok(id) => id,
            Err(_) => {
                conn.execute(
                    "INSERT INTO tags (name, created_at) VALUES (?1, ?2)",
                    params![t.label, now_ts()],
                )?;
                conn.last_insert_rowid()
            }
        };
        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id, source, created_at) \
             VALUES (?1, ?2, 'ai', ?3)",
            params![file_id, tag_id, now_ts()],
        )?;
    }

    Ok(tags)
}

// reqwest is already async. This stub exists so the `or_else` above type-checks.
fn futures_block_workaround() -> AppResult<Value> {
    Err(AppError::Other(anyhow::anyhow!(
        "no tool result and no fallback"
    )))
}

#[tauri::command]
pub async fn ai_classify_file(
    state: State<'_, AppState>,
    file_id: i64,
    categories: Vec<String>,
) -> AppResult<String> {
    let cli = client(&state)?;
    let path = lookup_path(&state, file_id)?;
    let p = Path::new(&path);
    let prompt = format!(
        "分類してください。次のカテゴリの中から 1 つだけ選んで JSON で回答:\n{}\n\nファイル: {}",
        categories.join(", "),
        p.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    );
    let schema = json!({
        "type": "object",
        "properties": { "category": { "type": "string" } },
        "required": ["category"]
    });
    let result = cli.tool_call(None, &prompt, "classify", schema).await?;
    Ok(result["category"]
        .as_str()
        .unwrap_or("unknown")
        .to_string())
}

#[tauri::command]
pub async fn ai_summarize(
    state: State<'_, AppState>,
    file_id: i64,
) -> AppResult<String> {
    let cli = client(&state)?;
    let path = lookup_path(&state, file_id)?;
    let p = Path::new(&path);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Phase 3.x: support PDFs via pdfium. For now: text-like files only.
    let text_exts = ["txt", "md", "log", "json", "yaml", "yml", "toml", "csv"];
    if !text_exts.contains(&ext) {
        return Err(AppError::Invalid(format!(
            "summarise not yet supported for .{ext}"
        )));
    }
    let body = std::fs::read_to_string(p)?;
    let body = if body.len() > 50_000 {
        format!("{}\n\n[...truncated, {} chars total]", &body[..50_000], body.len())
    } else {
        body
    };
    let prompt = format!(
        "次のテキストを日本語で 3 行で要約してください。\n\n---\n{body}"
    );
    let summary = cli
        .text(
            Some("You are a concise document summariser. Output Japanese."),
            &prompt,
        )
        .await?;

    let conn = state.db.get()?;
    conn.execute(
        "INSERT OR REPLACE INTO ai_summaries (file_id, summary, model, created_at) \
         VALUES (?1, ?2, ?3, ?4)",
        params![file_id, summary, VISION_MODEL, now_ts()],
    )?;
    Ok(summary)
}

#[tauri::command]
pub async fn ai_search(
    state: State<'_, AppState>,
    query: String,
) -> AppResult<Vec<AiSearchHit>> {
    let cli = client(&state)?;

    // Translate NL → SearchQuery via tool-use.
    let schema = json!({
        "type": "object",
        "properties": {
            "text": { "type": "string", "description": "free-text full-text query" },
            "extensions": {
                "type": "array",
                "items": { "type": "string" },
                "description": "lower-case extensions, no dot"
            },
            "modified_after_days_ago": {
                "type": "integer",
                "description": "if user mentions recency"
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" }
            }
        }
    });
    let now = now_ts();
    let system = "Translate Japanese/English natural-language file searches into a structured filter. \
                  Be conservative — leave fields null when unsure.";
    let parsed = cli
        .tool_call(
            Some(system),
            &format!("Query: {query}\n\nReturn the structured filter."),
            "search_filter",
            schema,
        )
        .await?;

    let mut q = SearchQuery::default();
    if let Some(t) = parsed["text"].as_str() {
        q.text = Some(t.to_string());
    }
    if let Some(arr) = parsed["extensions"].as_array() {
        q.extensions = Some(
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_lowercase()))
                .collect(),
        );
    }
    if let Some(d) = parsed["modified_after_days_ago"].as_i64() {
        q.modified_after = Some(now - d * 86_400);
    }

    // Resolve tag names → ids.
    if let Some(arr) = parsed["tags"].as_array() {
        let conn = state.db.get()?;
        let mut ids = Vec::new();
        for v in arr {
            if let Some(name) = v.as_str() {
                if let Ok(id) = conn.query_row(
                    "SELECT id FROM tags WHERE name = ?1",
                    [name],
                    |r| r.get::<_, i64>(0),
                ) {
                    ids.push(id);
                }
            }
        }
        if !ids.is_empty() {
            q.tag_ids = Some(ids);
        }
    }
    q.limit = Some(50);
    let result = indexer::run(&state.db, &q)?;
    let hits = result
        .items
        .into_iter()
        .map(|f| AiSearchHit {
            file_id: f.id.unwrap_or(-1),
            path: f.path.clone(),
            name: f.name.clone(),
            score: 1.0,
            reason: format!("{:?}", parsed),
        })
        .collect();
    Ok(hits)
}

#[tauri::command]
pub async fn ocr_file(
    state: State<'_, AppState>,
    file_id: i64,
) -> AppResult<String> {
    // Use Claude Vision as the OCR engine — saves us bundling Tesseract.
    let cli = client(&state)?;
    let path = lookup_path(&state, file_id)?;
    let p = Path::new(&path);
    let prompt =
        "この画像内のすべてのテキストを抽出してください。レイアウトはなるべく保ち、改行で区切る。テキストがなければ空文字。";
    let text = cli.vision(p, prompt).await?;

    let conn = state.db.get()?;
    conn.execute(
        "INSERT OR REPLACE INTO ocr_text (file_id, content, lang, created_at) \
         VALUES (?1, ?2, NULL, ?3)",
        params![file_id, text, now_ts()],
    )?;
    // Also push into FTS so `search_files` finds it.
    conn.execute(
        "INSERT OR REPLACE INTO files_fts(rowid, name, note) \
         SELECT id, name, COALESCE(note,'') || ' ' || ?1 FROM files WHERE id = ?2",
        params![text, file_id],
    )?;
    Ok(text)
}
