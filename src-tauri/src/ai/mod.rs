//! AI features — Phase 3.
//!
//! ## Backends
//! - Cloud: Claude API (Vision for images; chat for classification & NL search)
//! - Local: optional CLIP / Llava / Tesseract for users who prefer offline
//!
//! ## Surface
//! - `ai_tag_file` — image labels
//! - `ai_classify_file` — into a small set of buckets (config or learned)
//! - `ai_summarize` — short summary for documents
//! - `ai_search` — natural language → SearchQuery
//! - `ocr_file` — extract text from images / PDFs
//!
//! ## Storage
//! Tags written to `ai_tags` (with `model`, `confidence`).
//! Embeddings to `ai_embeddings`. OCR text to `ocr_text` and indexed via FTS.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AiTag {
    pub label: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize)]
pub struct AiSearchHit {
    pub file_id: i64,
    pub score: f32,
    pub reason: String,
}
