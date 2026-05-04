//! Live filesystem watcher.
//!
//! Wraps `notify-debouncer-mini`. On every batch of debounced events we look
//! up matching rules and run them. The handle owns the debouncer so dropping
//! it stops the watch.

use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::rules::engine;

#[allow(dead_code)]
pub struct WatcherHandle {
    debouncer: Debouncer<RecommendedWatcher>,
    pub roots: Vec<PathBuf>,
}

pub fn start(roots: Vec<PathBuf>, pool: DbPool) -> AppResult<WatcherHandle> {
    let pool_cb = pool.clone();
    let mut debouncer = new_debouncer(
        Duration::from_secs(1),
        move |res: DebounceEventResult| match res {
            Ok(events) => {
                for ev in events {
                    handle(&pool_cb, &ev.path);
                }
            }
            Err(e) => log::warn!("watch error: {e:?}"),
        },
    )?;

    for root in &roots {
        if root.exists() {
            debouncer
                .watcher()
                .watch(root, RecursiveMode::Recursive)?;
            log::info!("watching {}", root.display());
        }
    }

    Ok(WatcherHandle { debouncer, roots })
}

fn handle(pool: &DbPool, path: &Path) {
    if !path.is_file() {
        return;
    }
    let rules = match engine::rules_for_path(pool, path) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("rules_for_path: {e}");
            return;
        }
    };
    if rules.is_empty() {
        return;
    }
    let cand = match engine::Candidate::from_path(path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("candidate: {e}");
            return;
        }
    };
    for rule in rules {
        match engine::matches(pool, &cand, &rule) {
            Ok(true) => {
                let _ =
                    engine::execute(pool, &cand, &rule, &format!("rule:{}", rule.id));
                let _ = engine::touch_last_run(pool, rule.id);
            }
            Ok(false) => {}
            Err(e) => log::warn!("rule eval: {e}"),
        }
    }
}
