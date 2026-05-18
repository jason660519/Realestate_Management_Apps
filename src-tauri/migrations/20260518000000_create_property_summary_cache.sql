-- Cache surface for `list_property_summaries` (ADR-004 §2, refined by
-- property-document-boundary.md). The table holds *only* the server-projected
-- summary fields, not canonical evidence — it is safe to truncate at any time
-- and re-hydrate from the server.

CREATE TABLE IF NOT EXISTS property_summary_cache (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    address_raw TEXT,
    updated_at TEXT,
    last_synced_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS property_summary_cache_synced_at_idx
    ON property_summary_cache (last_synced_at DESC);
