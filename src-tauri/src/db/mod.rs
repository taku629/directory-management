//! SQLite connection pool and schema initialization.

pub mod schema;

use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::error::AppResult;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConn = r2d2::PooledConnection<SqliteConnectionManager>;

pub fn open_pool(path: &Path) -> AppResult<DbPool> {
    let manager = SqliteConnectionManager::file(path).with_init(|c| {
        c.pragma_update(None, "journal_mode", "WAL")?;
        c.pragma_update(None, "synchronous", "NORMAL")?;
        c.pragma_update(None, "foreign_keys", "ON")?;
        c.pragma_update(None, "temp_store", "MEMORY")?;
        Ok(())
    });
    let pool = Pool::builder().max_size(8).build(manager)?;
    Ok(pool)
}

pub fn init_schema(pool: &DbPool) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute_batch(schema::SCHEMA_SQL)?;
    log::info!("db schema initialized");
    Ok(())
}

pub fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}
