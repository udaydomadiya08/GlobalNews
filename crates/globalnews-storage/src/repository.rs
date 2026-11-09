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
                importance_score, domain_severity, geographic_reach, physical_impact, volume_dampener,
                confidence_score, independent_source_count, jurisdictional_diversity, has_sensor_validation, is_disputed,
                latitude, longitude, altitude_m, toponym_name, geonames_id, country_code, admin1_code,
                uncertainty_radius_km, status, first_seen_at, last_updated_at, citation_count
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22,
                ?23, ?24, ?25, ?26, ?27
            )
            ON CONFLICT(id) DO UPDATE SET
                macro_event_id = excluded.macro_event_id,
                title = excluded.title,
                summary = excluded.summary,
                importance_score = excluded.importance_score,
                domain_severity = excluded.domain_severity,
                geographic_reach = excluded.geographic_reach,
                physical_impact = excluded.physical_impact,
                volume_dampener = excluded.volume_dampener,
                confidence_score = excluded.confidence_score,
                independent_source_count = excluded.independent_source_count,
                jurisdictional_diversity = excluded.jurisdictional_diversity,
                has_sensor_validation = excluded.has_sensor_validation,
                is_disputed = excluded.is_disputed,
                latitude = excluded.latitude,
                longitude = excluded.longitude,
                altitude_m = excluded.altitude_m,
                toponym_name = excluded.toponym_name,
                geonames_id = excluded.geonames_id,
                country_code = excluded.country_code,
                admin1_code = excluded.admin1_code,
                uncertainty_radius_km = excluded.uncertainty_radius_km,
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
                event.importance.domain_severity,
                event.importance.geographic_reach,
                event.importance.physical_impact,
                event.importance.volume_dampener,
                event.confidence.final_confidence,
                event.confidence.independent_source_count,
                event.confidence.jurisdictional_diversity,
                if event.confidence.has_sensor_validation { 1 } else { 0 },
                if event.confidence.is_disputed { 1 } else { 0 },
                event.location.latitude,
                event.location.longitude,
                event.location.altitude_m,
                event.location.toponym_name,
                event.location.geonames_id,
                event.location.country_code,
                event.location.admin1_code,
                event.location.uncertainty_radius_km,
                serde_json::to_string(&event.status).unwrap().trim_matches('"'),
                event.first_seen_at,
                event.last_updated_at,
                event.citation_count,
            ],
        )?;

        Ok(())
    }

    pub fn get_micro_event(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<MicroEvent>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, macro_event_id, title, summary, category_code,
                    importance_score, domain_severity, geographic_reach, physical_impact, volume_dampener,
                    confidence_score, independent_source_count, jurisdictional_diversity, has_sensor_validation, is_disputed,
                    latitude, longitude, altitude_m, toponym_name, geonames_id, country_code, admin1_code,
                    uncertainty_radius_km, status, first_seen_at, last_updated_at, citation_count
             FROM micro_events WHERE id = ?1;",
        )?;

        let event = stmt
            .query_row(params![id], |row| {
                let status_str: String = row.get(23)?;
                let status: EventStatus = serde_json::from_str(&format!("\"{}\"", status_str))
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(23, rusqlite::types::Type::Text, Box::new(e)))?;

                let importance = ImportanceScore {
                    final_score: row.get(5)?,
                    domain_severity: row.get(6)?,
                    geographic_reach: row.get(7)?,
                    physical_impact: row.get(8)?,
                    volume_dampener: row.get(9)?,
                };

                let has_sensor_val: i32 = row.get(13)?;
                let is_disputed_val: i32 = row.get(14)?;

                let confidence = ConfidenceScore {
                    final_confidence: row.get(10)?,
                    independent_source_count: row.get(11)?,
                    jurisdictional_diversity: row.get(12)?,
                    has_sensor_validation: has_sensor_val != 0,
                    is_disputed: is_disputed_val != 0,
                };

                let location = GeographicLocation {
                    latitude: row.get(15)?,
                    longitude: row.get(16)?,
                    altitude_m: row.get(17)?,
                    toponym_name: row.get(18)?,
                    geonames_id: row.get(19)?,
                    country_code: row.get(20)?,
                    admin1_code: row.get(21)?,
                    uncertainty_radius_km: row.get(22)?,
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
                    first_seen_at: row.get(24)?,
                    last_updated_at: row.get(25)?,
                    citation_count: row.get(26)?,
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
    // Event Relationships CRUD
    // -------------------------------------------------------------------------
    pub fn insert_relationship(
        conn: &Connection,
        rel: &EventRelationship,
    ) -> Result<(), StorageError> {
        rel.validate()?;
        conn.execute(
            "INSERT INTO event_relationships (source_event_id, target_event_id, relationship_type, description)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(source_event_id, target_event_id, relationship_type) DO UPDATE SET
                 description = excluded.description;",
            params![
                rel.source_event_id,
                rel.target_event_id,
                serde_json::to_string(&rel.relationship_type).unwrap().trim_matches('"'),
                rel.description,
            ],
        )?;
        Ok(())
    }

    pub fn get_relationships_for_event(
        conn: &Connection,
        event_id: &str,
    ) -> Result<Vec<EventRelationship>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT source_event_id, target_event_id, relationship_type, description
             FROM event_relationships
             WHERE source_event_id = ?1 OR target_event_id = ?1;",
        )?;

        let rows = stmt.query_map(params![event_id], |row| {
            let rel_str: String = row.get(2)?;
            let rel_type: RelationType = serde_json::from_str(&format!("\"{}\"", rel_str))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?;

            Ok(EventRelationship {
                source_event_id: row.get(0)?,
                target_event_id: row.get(1)?,
                relationship_type: rel_type,
                description: row.get(3)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_micro_event(conn: &Connection, id: &str) -> Result<bool, StorageError> {
        let rows_affected = conn.execute(
            "DELETE FROM micro_events WHERE id = ?1;",
            params![id],
        )?;
        Ok(rows_affected > 0)
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
