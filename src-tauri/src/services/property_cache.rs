use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::{
    errors::AppError,
    models::{PropertyKind, PropertyStatus, PropertySummary},
};

/// Snapshot of cached property summaries plus the timestamp of the batch that
/// produced them. `last_synced_at` is `None` only when no batch has ever been
/// written (fresh install / cache wiped).
#[derive(Debug, Clone)]
pub struct CachedSummaries {
    pub rows: Vec<PropertySummary>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// Replace the cached batch with the given summaries. Wraps the work in a
/// transaction so a crash mid-write cannot leave a half-truncated table.
pub async fn replace_summaries(
    pool: &SqlitePool,
    rows: &[PropertySummary],
    synced_at: DateTime<Utc>,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(|error| AppError::LocalDb {
        message: format!("Failed to start cache transaction: {error}"),
    })?;

    sqlx::query("DELETE FROM property_summary_cache")
        .execute(&mut *tx)
        .await
        .map_err(|error| AppError::LocalDb {
            message: format!("Failed to clear property_summary_cache: {error}"),
        })?;

    let synced_at_string = synced_at.to_rfc3339();
    for row in rows {
        sqlx::query(
            "INSERT INTO property_summary_cache \
             (id, display_name, kind, status, address_raw, updated_at, last_synced_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&row.id)
        .bind(&row.display_name)
        .bind(serde_kind(&row.kind))
        .bind(serde_status(&row.status))
        .bind(row.address_raw.as_deref())
        .bind(row.updated_at.map(|dt| dt.to_rfc3339()))
        .bind(&synced_at_string)
        .execute(&mut *tx)
        .await
        .map_err(|error| AppError::LocalDb {
            message: format!("Failed to insert cache row for {}: {error}", row.id),
        })?;
    }

    tx.commit().await.map_err(|error| AppError::LocalDb {
        message: format!("Failed to commit cache transaction: {error}"),
    })?;

    Ok(())
}

/// Read every cached summary. Returns rows ordered by the server-provided
/// `updated_at` descending so the offline view matches what would come back
/// from a fresh PostgREST query.
pub async fn read_summaries(pool: &SqlitePool) -> Result<CachedSummaries, AppError> {
    let raw_rows = sqlx::query_as::<_, CacheRow>(
        "SELECT id, display_name, kind, status, address_raw, updated_at, last_synced_at \
         FROM property_summary_cache \
         ORDER BY updated_at DESC NULLS LAST",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| AppError::LocalDb {
        message: format!("Failed to read property_summary_cache: {error}"),
    })?;

    let last_synced_at = raw_rows
        .iter()
        .filter_map(|row| parse_rfc3339(&row.last_synced_at))
        .max();

    let rows = raw_rows.into_iter().map(PropertySummary::from).collect();

    Ok(CachedSummaries {
        rows,
        last_synced_at,
    })
}

#[derive(sqlx::FromRow)]
struct CacheRow {
    id: String,
    display_name: String,
    kind: String,
    status: String,
    address_raw: Option<String>,
    updated_at: Option<String>,
    last_synced_at: String,
}

impl From<CacheRow> for PropertySummary {
    fn from(row: CacheRow) -> Self {
        PropertySummary {
            id: row.id,
            display_name: row.display_name,
            kind: deserialize_kind(&row.kind),
            status: deserialize_status(&row.status),
            address_raw: row.address_raw,
            updated_at: row.updated_at.as_deref().and_then(parse_rfc3339),
        }
    }
}

fn parse_rfc3339(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn serde_kind(kind: &PropertyKind) -> &'static str {
    match kind {
        PropertyKind::Sale => "sale",
        PropertyKind::Rental => "rental",
        PropertyKind::LandOnly => "land_only",
        PropertyKind::Commercial => "commercial",
        PropertyKind::Unknown => "unknown",
    }
}

fn serde_status(status: &PropertyStatus) -> &'static str {
    match status {
        PropertyStatus::Draft => "draft",
        PropertyStatus::Active => "active",
        PropertyStatus::Pending => "pending",
        PropertyStatus::Archived => "archived",
        PropertyStatus::Unknown => "unknown",
    }
}

fn deserialize_kind(value: &str) -> PropertyKind {
    match value {
        "sale" => PropertyKind::Sale,
        "rental" => PropertyKind::Rental,
        "land_only" => PropertyKind::LandOnly,
        "commercial" => PropertyKind::Commercial,
        _ => PropertyKind::Unknown,
    }
}

fn deserialize_status(value: &str) -> PropertyStatus {
    match value {
        "draft" => PropertyStatus::Draft,
        "active" => PropertyStatus::Active,
        "pending" => PropertyStatus::Pending,
        "archived" => PropertyStatus::Archived,
        _ => PropertyStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::local_db;

    fn sample_row(id: &str, name: &str, kind: PropertyKind) -> PropertySummary {
        PropertySummary {
            id: id.to_string(),
            display_name: name.to_string(),
            kind,
            status: PropertyStatus::Active,
            address_raw: Some(format!("address for {name}")),
            updated_at: DateTime::parse_from_rfc3339("2026-05-10T12:00:00Z")
                .ok()
                .map(|dt| dt.with_timezone(&Utc)),
        }
    }

    #[tokio::test]
    async fn replace_then_read_round_trips() {
        let pool = local_db::open_in_memory().await;
        let rows = vec![
            sample_row("11", "Sale Sample", PropertyKind::Sale),
            sample_row("22", "Rental Sample", PropertyKind::Rental),
        ];
        let synced = Utc::now();

        replace_summaries(&pool, &rows, synced).await.unwrap();
        let cached = read_summaries(&pool).await.unwrap();

        assert_eq!(cached.rows.len(), 2);
        assert!(cached.rows.iter().any(|row| row.kind == PropertyKind::Sale));
        assert!(cached.last_synced_at.is_some());
    }

    #[tokio::test]
    async fn replace_truncates_previous_batch() {
        let pool = local_db::open_in_memory().await;
        replace_summaries(
            &pool,
            &[sample_row("11", "Old", PropertyKind::Sale)],
            Utc::now(),
        )
        .await
        .unwrap();
        replace_summaries(
            &pool,
            &[sample_row("22", "Fresh", PropertyKind::Rental)],
            Utc::now(),
        )
        .await
        .unwrap();

        let cached = read_summaries(&pool).await.unwrap();
        assert_eq!(cached.rows.len(), 1);
        assert_eq!(cached.rows[0].display_name, "Fresh");
    }

    #[tokio::test]
    async fn unknown_kind_falls_back_when_stored_value_drifts() {
        let pool = local_db::open_in_memory().await;
        sqlx::query(
            "INSERT INTO property_summary_cache \
             (id, display_name, kind, status, address_raw, updated_at, last_synced_at) \
             VALUES ('33', 'Drifted', 'mixed_use', 'pending', NULL, NULL, ?)",
        )
        .bind(Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();

        let cached = read_summaries(&pool).await.unwrap();
        assert_eq!(cached.rows.len(), 1);
        assert_eq!(cached.rows[0].kind, PropertyKind::Unknown);
        assert_eq!(cached.rows[0].status, PropertyStatus::Pending);
    }

    #[tokio::test]
    async fn empty_cache_returns_no_rows_and_no_synced_at() {
        let pool = local_db::open_in_memory().await;
        let cached = read_summaries(&pool).await.unwrap();
        assert!(cached.rows.is_empty());
        assert!(cached.last_synced_at.is_none());
    }
}
