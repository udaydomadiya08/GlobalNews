use crate::error::StorageError;
use globalnews_contracts::*;
use rusqlite::{params, Connection, OptionalExtension};

pub struct Repository;

impl Repository {
    // -------------------------------------------------------------------------
    // Macro Events CRUD
    // -------------------------------------------------------------------------
    pub fn insert_macro_event(conn: &Connection, event: &MacroEvent) -> Result<(), StorageError> {
        event.validate()?;
        conn.execute(
            "INSERT INTO macro_events (
                id, title, category_code, summary, status,
                min_lat, min_lon, max_lat, max_lon, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                summary = excluded.summary,
                status = excluded.status,
                updated_at = excluded.updated_at;",
            params![
                event.id,
                event.title,
                event.category_code,
                event.summary,
                serde_json::to_string(&event.status).unwrap().trim_matches('"'),
                event.bounding_box[0],
                event.bounding_box[1],
                event.bounding_box[2],
                event.bounding_box[3],
                event.created_at,
                event.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_macro_event(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<MacroEvent>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, title, category_code, summary, status,
                    min_lat, min_lon, max_lat, max_lon, created_at, updated_at
             FROM macro_events WHERE id = ?1;",
        )?;

        let event = stmt
            .query_row(params![id], |row| {
                let status_str: String = row.get(4)?;
                let status: MacroEventStatus = serde_json::from_str(&format!("\"{}\"", status_str))
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;

                Ok(MacroEvent {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    category_code: row.get(2)?,
                    summary: row.get(3)?,
                    status,
                    bounding_box: [row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?],
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    active_micro_event_count: 0,
                })
            })
            .optional()?;

        Ok(event)
    }

    // -------------------------------------------------------------------------
    // Micro Events CRUD
    // -------------------------------------------------------------------------
    pub fn insert_micro_event(conn: &Connection, event: &MicroEvent) -> Result<(), StorageError> {
        event.validate()?;
        conn.execute(
            "INSERT INTO micro_events (
                id, macro_event_id, title, summary, category_code,
                importance_score, confidence_score, latitude, longitude,
                uncertainty_radius_km, status, first_seen_at, last_updated_at, citation_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(id) DO UPDATE SET
                macro_event_id = excluded.macro_event_id,
                title = excluded.title,
                summary = excluded.summary,
                importance_score = excluded.importance_score,
                confidence_score = excluded.confidence_score,
                status = excluded.status,
                last_updated_at = excluded.last_updated_at,
                citation_count = excluded.citation_count;",
            params![
                event.id,
                event.macro_event_id,
                event.title,
                event.summary,
                event.category_code,
                event.importance.final_score,
                event.confidence.final_confidence,
                event.location.latitude,
                event.location.longitude,
                event.location.uncertainty_radius_km,
                serde_json::to_string(&event.status).unwrap().trim_matches('"'),
                event.first_seen_at,
                event.last_updated_at,
                event.citation_count,
            ],
        )?;

        // Update FTS5 virtual index
        conn.execute(
            "INSERT INTO fts_events(rowid, title, summary)
             SELECT rowid, title, summary FROM micro_events WHERE id = ?1;",
            params![event.id],
        )?;

        Ok(())
    }

    pub fn get_micro_event(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<MicroEvent>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, macro_event_id, title, summary, category_code,
                    importance_score, confidence_score, latitude, longitude,
                    uncertainty_radius_km, status, first_seen_at, last_updated_at, citation_count
             FROM micro_events WHERE id = ?1;",
        )?;

        let event = stmt
            .query_row(params![id], |row| {
                let status_str: String = row.get(10)?;
                let status: EventStatus = serde_json::from_str(&format!("\"{}\"", status_str))
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(10, rusqlite::types::Type::Text, Box::new(e)))?;

                let importance_val: f64 = row.get(5)?;
                let confidence_val: f64 = row.get(6)?;
                let lat: f64 = row.get(7)?;
                let lon: f64 = row.get(8)?;
                let uncertainty: f64 = row.get(9)?;

                let location = GeographicLocation {
                    latitude: lat,
                    longitude: lon,
                    altitude_m: None,
                    toponym_name: "Resolved Coordinate".into(),
                    geonames_id: None,
                    country_code: "XX".into(),
                    admin1_code: None,
                    uncertainty_radius_km: uncertainty,
                };

                let importance = ImportanceScore {
                    final_score: importance_val,
                    domain_severity: importance_val,
                    geographic_reach: 5.0,
                    physical_impact: 5.0,
                    volume_dampener: 1.0,
                };

                let confidence = ConfidenceScore {
                    final_confidence: confidence_val,
                    independent_source_count: 1,
                    jurisdictional_diversity: 0.5,
                    has_sensor_validation: false,
                    is_disputed: false,
                };

                Ok(MicroEvent {
                    id: row.get(0)?,
                    macro_event_id: row.get(1)?,
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    category_code: row.get(4)?,
                    importance,
                    confidence,
                    location,
                    status,
                    first_seen_at: row.get(11)?,
                    last_updated_at: row.get(12)?,
                    citation_count: row.get(13)?,
                })
            })
            .optional()?;

        Ok(event)
    }

    // -------------------------------------------------------------------------
    // Articles CRUD
    // -------------------------------------------------------------------------
    pub fn insert_article(conn: &Connection, article: &Article) -> Result<(), StorageError> {
        article.validate()?;
        conn.execute(
            "INSERT INTO articles (
                id, micro_event_id, url, title, body_excerpt,
                language, source_id, content_hash, published_at, ingested_at, is_syndicated
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(content_hash) DO NOTHING;",
            params![
                article.id,
                article.micro_event_id,
                article.url,
                article.title,
                article.body_excerpt,
                article.language,
                article.source_id,
                article.content_hash,
                article.published_at,
                article.ingested_at,
                if article.is_syndicated { 1 } else { 0 },
            ],
        )?;

        // Update FTS5 index for articles
        conn.execute(
            "INSERT INTO fts_articles(rowid, title, body_excerpt)
             SELECT rowid, title, body_excerpt FROM articles WHERE id = ?1;",
            params![article.id],
        )?;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Sources CRUD
    // -------------------------------------------------------------------------
    pub fn insert_source(conn: &Connection, source: &Source) -> Result<(), StorageError> {
        source.validate()?;
        conn.execute(
            "INSERT INTO sources (
                id, name, tier, feed_url, feed_type, reliability_score,
                etag, last_modified, last_polled_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                etag = excluded.etag,
                last_modified = excluded.last_modified,
                last_polled_at = excluded.last_polled_at;",
            params![
                source.id,
                source.name,
                source.tier,
                source.feed_url,
                source.feed_type,
                source.reliability_score,
                source.etag,
                source.last_modified,
                source.last_polled_at,
            ],
        )?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Full-Text Search (FTS5 BM25 Ranking)
    // -------------------------------------------------------------------------
    pub fn search_events(
        conn: &Connection,
        query: &str,
        limit: u32,
    ) -> Result<Vec<(String, String, f64)>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT m.id, m.title, bm25(fts_events) as rank
             FROM fts_events f
             JOIN micro_events m ON f.rowid = m.rowid
             WHERE fts_events MATCH ?1
             ORDER BY rank
             LIMIT ?2;",
        )?;

        let rows = stmt.query_map(params![query, limit], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }
}
