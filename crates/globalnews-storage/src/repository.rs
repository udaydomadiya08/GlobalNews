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
                serde_json::to_string(&event.status)
                    .unwrap()
                    .trim_matches('"'),
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
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

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
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            23,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

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
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

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
        let rows_affected = conn.execute("DELETE FROM micro_events WHERE id = ?1;", params![id])?;
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

    // -------------------------------------------------------------------------
    // Phase 1.2: Feed Ingestion & Normalization Repository Methods
    // -------------------------------------------------------------------------

    pub fn upsert_source(conn: &Connection, source: &Source) -> Result<(), StorageError> {
        source.validate()?;
        let langs_json = serde_json::to_string(&source.supported_languages).unwrap_or_else(|_| "[]".into());
        let cat_str = source.category.as_str();

        conn.execute(
            "INSERT INTO sources (
                id, name, tier, feed_url, feed_type, reliability_score,
                etag, last_modified, last_polled_at, canonical_domain, category,
                supported_languages, geographic_coverage, attribution_requirements,
                licence_policy_url, enabled, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, datetime('now'), datetime('now'))
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                tier = excluded.tier,
                feed_url = excluded.feed_url,
                feed_type = excluded.feed_type,
                reliability_score = excluded.reliability_score,
                etag = excluded.etag,
                last_modified = excluded.last_modified,
                last_polled_at = excluded.last_polled_at,
                canonical_domain = excluded.canonical_domain,
                category = excluded.category,
                supported_languages = excluded.supported_languages,
                geographic_coverage = excluded.geographic_coverage,
                attribution_requirements = excluded.attribution_requirements,
                licence_policy_url = excluded.licence_policy_url,
                enabled = excluded.enabled,
                updated_at = datetime('now');",
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
                source.canonical_domain,
                cat_str,
                langs_json,
                source.geographic_coverage,
                source.attribution_requirements,
                source.licence_policy_url,
                if source.enabled { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }

    pub fn get_source(conn: &Connection, id: &str) -> Result<Option<Source>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, name, tier, feed_url, feed_type, reliability_score,
                    etag, last_modified, last_polled_at, canonical_domain, category,
                    supported_languages, geographic_coverage, attribution_requirements,
                    licence_policy_url, enabled
             FROM sources WHERE id = ?1;",
        )?;

        let source = stmt
            .query_row(params![id], |row| {
                let langs_str: String = row.get(11)?;
                let langs: Vec<String> = serde_json::from_str(&langs_str).unwrap_or_else(|_| vec!["en".into()]);
                let cat_str: String = row.get(10)?;
                let enabled_int: i32 = row.get(15)?;

                Ok(Source {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    tier: row.get(2)?,
                    feed_url: row.get(3)?,
                    feed_type: row.get(4)?,
                    reliability_score: row.get(5)?,
                    etag: row.get(6)?,
                    last_modified: row.get(7)?,
                    last_polled_at: row.get(8)?,
                    canonical_domain: row.get(9)?,
                    category: SourceCategory::parse_str(&cat_str),
                    supported_languages: langs,
                    geographic_coverage: row.get(12)?,
                    attribution_requirements: row.get(13)?,
                    licence_policy_url: row.get(14)?,
                    enabled: enabled_int != 0,
                })
            })
            .optional()?;

        Ok(source)
    }

    pub fn list_sources(conn: &Connection) -> Result<Vec<Source>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, name, tier, feed_url, feed_type, reliability_score,
                    etag, last_modified, last_polled_at, canonical_domain, category,
                    supported_languages, geographic_coverage, attribution_requirements,
                    licence_policy_url, enabled
             FROM sources ORDER BY tier ASC, name ASC;",
        )?;

        let rows = stmt.query_map([], |row| {
            let langs_str: String = row.get(11)?;
            let langs: Vec<String> = serde_json::from_str(&langs_str).unwrap_or_else(|_| vec!["en".into()]);
            let cat_str: String = row.get(10)?;
            let enabled_int: i32 = row.get(15)?;

            Ok(Source {
                id: row.get(0)?,
                name: row.get(1)?,
                tier: row.get(2)?,
                feed_url: row.get(3)?,
                feed_type: row.get(4)?,
                reliability_score: row.get(5)?,
                etag: row.get(6)?,
                last_modified: row.get(7)?,
                last_polled_at: row.get(8)?,
                canonical_domain: row.get(9)?,
                category: SourceCategory::parse_str(&cat_str),
                supported_languages: langs,
                geographic_coverage: row.get(12)?,
                attribution_requirements: row.get(13)?,
                licence_policy_url: row.get(14)?,
                enabled: enabled_int != 0,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------------------
    // Feed Endpoints CRUD
    // -------------------------------------------------------------------------

    pub fn upsert_feed_endpoint(conn: &Connection, ep: &FeedEndpoint) -> Result<(), StorageError> {
        ep.validate()?;
        conn.execute(
            "INSERT INTO feed_endpoints (
                id, source_id, endpoint_url, feed_format, retrieval_method,
                polling_interval_secs, rate_limit_per_minute, enabled, priority,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                endpoint_url = excluded.endpoint_url,
                feed_format = excluded.feed_format,
                retrieval_method = excluded.retrieval_method,
                polling_interval_secs = excluded.polling_interval_secs,
                rate_limit_per_minute = excluded.rate_limit_per_minute,
                enabled = excluded.enabled,
                priority = excluded.priority,
                updated_at = excluded.updated_at;",
            params![
                ep.id,
                ep.source_id,
                ep.endpoint_url,
                ep.feed_format.as_str(),
                ep.retrieval_method.as_str(),
                ep.polling_interval_secs,
                ep.rate_limit_per_minute,
                if ep.enabled { 1 } else { 0 },
                ep.priority,
                ep.created_at,
                ep.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_feed_endpoint(conn: &Connection, id: &str) -> Result<Option<FeedEndpoint>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, source_id, endpoint_url, feed_format, retrieval_method,
                    polling_interval_secs, rate_limit_per_minute, enabled, priority,
                    created_at, updated_at
             FROM feed_endpoints WHERE id = ?1;",
        )?;

        let endpoint = stmt
            .query_row(params![id], |row| {
                let fmt_str: String = row.get(3)?;
                let ret_str: String = row.get(4)?;
                let enabled_int: i32 = row.get(7)?;

                Ok(FeedEndpoint {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    endpoint_url: row.get(2)?,
                    feed_format: FeedFormat::parse_str(&fmt_str).unwrap_or(FeedFormat::Rss2),
                    retrieval_method: RetrievalMethod::parse_str(&ret_str),
                    polling_interval_secs: row.get(5)?,
                    rate_limit_per_minute: row.get(6)?,
                    enabled: enabled_int != 0,
                    priority: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .optional()?;

        Ok(endpoint)
    }

    pub fn list_feed_endpoints(conn: &Connection) -> Result<Vec<FeedEndpoint>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, source_id, endpoint_url, feed_format, retrieval_method,
                    polling_interval_secs, rate_limit_per_minute, enabled, priority,
                    created_at, updated_at
             FROM feed_endpoints ORDER BY priority ASC, id ASC;",
        )?;

        let rows = stmt.query_map([], |row| {
            let fmt_str: String = row.get(3)?;
            let ret_str: String = row.get(4)?;
            let enabled_int: i32 = row.get(7)?;

            Ok(FeedEndpoint {
                id: row.get(0)?,
                source_id: row.get(1)?,
                endpoint_url: row.get(2)?,
                feed_format: FeedFormat::parse_str(&fmt_str).unwrap_or(FeedFormat::Rss2),
                retrieval_method: RetrievalMethod::parse_str(&ret_str),
                polling_interval_secs: row.get(5)?,
                rate_limit_per_minute: row.get(6)?,
                enabled: enabled_int != 0,
                priority: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------------------
    // Feed Checkpoints
    // -------------------------------------------------------------------------

    pub fn get_feed_checkpoint(conn: &Connection, endpoint_id: &str) -> Result<Option<FeedCheckpoint>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT endpoint_id, etag, last_modified, last_retrieval_at,
                    last_status_code, last_successful_article_count, content_hash
             FROM feed_checkpoints WHERE endpoint_id = ?1;",
        )?;

        let cp = stmt
            .query_row(params![endpoint_id], |row| {
                Ok(FeedCheckpoint {
                    endpoint_id: row.get(0)?,
                    etag: row.get(1)?,
                    last_modified: row.get(2)?,
                    last_retrieval_at: row.get(3)?,
                    last_status_code: row.get(4)?,
                    last_successful_article_count: row.get(5)?,
                    content_hash: row.get(6)?,
                })
            })
            .optional()?;

        Ok(cp)
    }

    pub fn upsert_feed_checkpoint(conn: &Connection, cp: &FeedCheckpoint) -> Result<(), StorageError> {
        conn.execute(
            "INSERT INTO feed_checkpoints (
                endpoint_id, etag, last_modified, last_retrieval_at,
                last_status_code, last_successful_article_count, content_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(endpoint_id) DO UPDATE SET
                etag = excluded.etag,
                last_modified = excluded.last_modified,
                last_retrieval_at = excluded.last_retrieval_at,
                last_status_code = excluded.last_status_code,
                last_successful_article_count = excluded.last_successful_article_count,
                content_hash = excluded.content_hash;",
            params![
                cp.endpoint_id,
                cp.etag,
                cp.last_modified,
                cp.last_retrieval_at,
                cp.last_status_code,
                cp.last_successful_article_count,
                cp.content_hash,
            ],
        )?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Normalized Articles & Deduplicating Ingestion
    // -------------------------------------------------------------------------

    pub fn insert_normalized_article_if_unique(
        conn: &mut Connection,
        article: &NormalizedArticle,
    ) -> Result<InsertResult, StorageError> {
        article.validate()?;

        let tx = conn.transaction()?;

        // 1. Check exact canonical URL duplicate
        let existing_canon: Option<String> = tx
            .query_row(
                "SELECT article_id FROM article_fingerprints WHERE fingerprint_hash = ?1;",
                params![article.canonical_url_hash],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(existing_id) = existing_canon {
            return Ok(InsertResult::Duplicate {
                existing_id,
                reason: "canonical_url_duplicate".into(),
            });
        }

        // 2. Check title+date duplicate for same publisher
        let existing_title_date: Option<String> = tx
            .query_row(
                "SELECT article_id FROM article_fingerprints WHERE fingerprint_hash = ?1 AND source_id = ?2;",
                params![article.title_date_hash, article.source_id],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(existing_id) = existing_title_date {
            return Ok(InsertResult::Duplicate {
                existing_id,
                reason: "publisher_title_date_duplicate".into(),
            });
        }

        // 3. Persist normalized article
        let authors_json = serde_json::to_string(&article.authors).unwrap_or_else(|_| "[]".into());
        let cats_json = serde_json::to_string(&article.feed_categories).unwrap_or_else(|_| "[]".into());

        tx.execute(
            "INSERT INTO normalized_articles (
                id, source_id, endpoint_id, canonical_url, original_url,
                title, description_excerpt, content_classification,
                published_at, raw_published_at, retrieved_at, updated_at,
                language, authors, feed_categories, source_provenance,
                is_syndicated, canonical_url_hash, title_date_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19);",
            params![
                article.id,
                article.source_id,
                article.endpoint_id,
                article.canonical_url,
                article.original_url,
                article.title,
                article.description_excerpt,
                article.content_classification.as_str(),
                article.published_at,
                article.raw_published_at,
                article.retrieved_at,
                article.updated_at,
                article.language,
                authors_json,
                cats_json,
                article.source_provenance,
                if article.is_syndicated { 1 } else { 0 },
                article.canonical_url_hash,
                article.title_date_hash,
            ],
        )?;

        // 4. Record fingerprints
        tx.execute(
            "INSERT INTO article_fingerprints (fingerprint_hash, hash_type, article_id, source_id, first_seen_at)
             VALUES (?1, 'canonical_url', ?2, ?3, ?4);",
            params![article.canonical_url_hash, article.id, article.source_id, article.retrieved_at],
        )?;

        tx.execute(
            "INSERT INTO article_fingerprints (fingerprint_hash, hash_type, article_id, source_id, first_seen_at)
             VALUES (?1, 'title_date', ?2, ?3, ?4);",
            params![article.title_date_hash, article.id, article.source_id, article.retrieved_at],
        )?;

        tx.commit()?;
        Ok(InsertResult::Inserted(article.id.clone()))
    }

    pub fn get_normalized_article(conn: &Connection, id: &str) -> Result<Option<NormalizedArticle>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, source_id, endpoint_id, canonical_url, original_url,
                    title, description_excerpt, content_classification,
                    published_at, raw_published_at, retrieved_at, updated_at,
                    language, authors, feed_categories, source_provenance,
                    is_syndicated, canonical_url_hash, title_date_hash
             FROM normalized_articles WHERE id = ?1;",
        )?;

        let article = stmt
            .query_row(params![id], |row| {
                let class_str: String = row.get(7)?;
                let authors_str: String = row.get(13)?;
                let cats_str: String = row.get(14)?;
                let syn_int: i32 = row.get(16)?;

                Ok(NormalizedArticle {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    endpoint_id: row.get(2)?,
                    canonical_url: row.get(3)?,
                    original_url: row.get(4)?,
                    title: row.get(5)?,
                    description_excerpt: row.get(6)?,
                    content_classification: ContentStorageClassification::parse_str(&class_str),
                    published_at: row.get(8)?,
                    raw_published_at: row.get(9)?,
                    retrieved_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    language: row.get(12)?,
                    authors: serde_json::from_str(&authors_str).unwrap_or_default(),
                    feed_categories: serde_json::from_str(&cats_str).unwrap_or_default(),
                    source_provenance: row.get(15)?,
                    is_syndicated: syn_int != 0,
                    canonical_url_hash: row.get(17)?,
                    title_date_hash: row.get(18)?,
                })
            })
            .optional()?;

        Ok(article)
    }

    pub fn search_normalized_articles_fts(
        conn: &Connection,
        query: &str,
        limit: u32,
    ) -> Result<Vec<(String, String, f64)>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.title, bm25(normalized_articles_fts) as rank
             FROM normalized_articles_fts f
             JOIN normalized_articles a ON f.rowid = a.rowid
             WHERE normalized_articles_fts MATCH ?1
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

    // -------------------------------------------------------------------------
    // Audit Runs, Error Logs, and Health Metrics
    // -------------------------------------------------------------------------

    pub fn record_ingestion_run(conn: &Connection, run: &IngestionRun) -> Result<(), StorageError> {
        run.validate()?;
        conn.execute(
            "INSERT INTO ingestion_runs (
                id, endpoint_id, started_at, completed_at, status,
                http_status_code, articles_fetched, articles_new,
                articles_duplicate, error_message
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
            params![
                run.id,
                run.endpoint_id,
                run.started_at,
                run.completed_at,
                run.status.as_str(),
                run.http_status_code,
                run.articles_fetched,
                run.articles_new,
                run.articles_duplicate,
                run.error_message,
            ],
        )?;
        Ok(())
    }

    pub fn record_retrieval_error(conn: &Connection, err: &RetrievalError) -> Result<(), StorageError> {
        conn.execute(
            "INSERT INTO retrieval_errors (
                id, run_id, endpoint_id, error_kind, details, occurred_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            params![
                err.id,
                err.run_id,
                err.endpoint_id,
                err.error_kind.as_str(),
                err.details,
                err.occurred_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_source_health(conn: &Connection, source_id: &str) -> Result<Option<SourceHealth>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT source_id, total_requests, successful_requests, failed_requests,
                    rate_limited_count, consecutive_failures, backoff_until,
                    last_error_message, last_error_at, reliability_score, updated_at
             FROM source_health WHERE source_id = ?1;",
        )?;

        let health = stmt
            .query_row(params![source_id], |row| {
                Ok(SourceHealth {
                    source_id: row.get(0)?,
                    total_requests: row.get(1)?,
                    successful_requests: row.get(2)?,
                    failed_requests: row.get(3)?,
                    rate_limited_count: row.get(4)?,
                    consecutive_failures: row.get(5)?,
                    backoff_until: row.get(6)?,
                    last_error_message: row.get(7)?,
                    last_error_at: row.get(8)?,
                    reliability_score: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .optional()?;

        Ok(health)
    }

    pub fn update_source_health(conn: &Connection, health: &SourceHealth) -> Result<(), StorageError> {
        conn.execute(
            "INSERT INTO source_health (
                source_id, total_requests, successful_requests, failed_requests,
                rate_limited_count, consecutive_failures, backoff_until,
                last_error_message, last_error_at, reliability_score, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(source_id) DO UPDATE SET
                total_requests = excluded.total_requests,
                successful_requests = excluded.successful_requests,
                failed_requests = excluded.failed_requests,
                rate_limited_count = excluded.rate_limited_count,
                consecutive_failures = excluded.consecutive_failures,
                backoff_until = excluded.backoff_until,
                last_error_message = excluded.last_error_message,
                last_error_at = excluded.last_error_at,
                reliability_score = excluded.reliability_score,
                updated_at = excluded.updated_at;",
            params![
                health.source_id,
                health.total_requests,
                health.successful_requests,
                health.failed_requests,
                health.rate_limited_count,
                health.consecutive_failures,
                health.backoff_until,
                health.last_error_message,
                health.last_error_at,
                health.reliability_score,
                health.updated_at,
            ],
        )?;
        Ok(())
    }
}
