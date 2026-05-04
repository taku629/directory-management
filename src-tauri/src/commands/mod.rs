//! Tauri IPC command handlers.
//!
//! Each submodule maps 1:1 with a feature area. Phase 1 commands are fully
//! implemented; Phase 2-4 modules are wired so the frontend can call them
//! today and receive `LockedFeature` / `NotImplemented` errors with stable
//! response shapes — turning them on later is a matter of filling in
//! implementations behind the existing signatures.

pub mod ai;
pub mod cloud;
pub mod disk;
pub mod duplicates;
pub mod files;
pub mod history;
pub mod license;
pub mod rules;
pub mod search;
pub mod settings;
pub mod tags;
pub mod watcher;
