//! Filesystem watcher (Phase 2: live indexing + rule triggers).
//!
//! Phase 1 provides the structure and a no-op handle so the rest of the app
//! can wire it up. Activation lands with the rules engine.

use std::path::PathBuf;

#[allow(dead_code)]
pub struct WatcherHandle {
    pub roots: Vec<PathBuf>,
    // notify debouncer is held here once Phase 2 turns the watcher on
}

impl WatcherHandle {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self { roots }
    }
}
