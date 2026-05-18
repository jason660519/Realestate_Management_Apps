use std::path::PathBuf;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::errors::AppError;

const STATE_FILE_NAME: &str = "state.db";

// Embed migrations from src-tauri/migrations/ at compile time. ADR-004 § Migration
// originally proposed `refinery`; the implementation uses sqlx's own `migrate!`
// macro because we already pull sqlx in, the embedded path is well documented,
// and it keeps the runtime dependency surface smaller.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Open (or create) the local SQLite state database under the Tauri app data
/// directory and run any pending migrations. Always safe to call at startup —
/// missing parent directory is created, missing schema is applied idempotently.
pub async fn open(app_data_dir: PathBuf) -> Result<SqlitePool, AppError> {
    std::fs::create_dir_all(&app_data_dir).map_err(|error| AppError::LocalDb {
        message: format!(
            "Failed to create local state directory {}: {error}",
            app_data_dir.display(),
        ),
    })?;

    let db_path = app_data_dir.join(STATE_FILE_NAME);
    let url = format!("sqlite://{}", db_path.display());

    let options = SqliteConnectOptions::from_str(&url)
        .map_err(|error| AppError::LocalDb {
            message: format!("Invalid SQLite URL {url}: {error}"),
        })?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .map_err(|error| AppError::LocalDb {
            message: format!("Failed to open local SQLite at {url}: {error}"),
        })?;

    MIGRATOR
        .run(&pool)
        .await
        .map_err(|error| AppError::LocalDb {
            message: format!("Failed to apply local SQLite migrations: {error}"),
        })?;

    Ok(pool)
}

/// Test helper that opens an in-memory SQLite pool with migrations applied.
#[cfg(test)]
pub async fn open_in_memory() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("in-memory sqlite url should be valid")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("in-memory sqlite pool should connect");

    MIGRATOR
        .run(&pool)
        .await
        .expect("in-memory migrations should apply");

    pool
}
