use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{AppHandle, Manager};

use crate::db;
use crate::error::AppResult;
use crate::fs::watcher::WatcherHandle;

/// Application-wide state managed by Tauri (`tauri::Manager`).
pub struct AppState {
    pub db: db::DbPool,
    pub data_dir: PathBuf,
    pub watcher: Arc<Mutex<Option<WatcherHandle>>>,
}

impl AppState {
    pub fn init(app: &AppHandle) -> AppResult<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| crate::error::AppError::Other(anyhow::anyhow!(e)))?;
        std::fs::create_dir_all(&data_dir)?;
        log::info!("data dir: {}", data_dir.display());

        let db_path = data_dir.join("sift.db");
        let pool = db::open_pool(&db_path)?;
        db::init_schema(&pool)?;

        Ok(Self {
            db: pool,
            data_dir,
            watcher: Arc::new(Mutex::new(None)),
        })
    }
}
