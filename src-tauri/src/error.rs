use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("db: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("pool: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("walkdir: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("notify: {0}")]
    Notify(#[from] notify::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    Invalid(String),

    #[error("locked feature: {0} requires the {1} tier")]
    LockedFeature(&'static str, &'static str),

    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
