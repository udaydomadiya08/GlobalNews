use super::adapter::{AtomAdapter, FeedAdapter, Rss2Adapter, UsgsEarthquakeAdapter};
use super::error::IngestionError;
use super::normalizer::ArticleNormalizer;
use super::resilience::BackoffPolicy;
use super::transport::{HttpRequest, HttpResponse, HttpTransport};
use chrono::Utc;
use globalnews_contracts::{
    ErrorKind, FeedCheckpoint, FeedEndpoint, FeedFormat, IngestionRun, IngestionStatus,
    InsertResult, RetrievalError, Source, SourceHealth,
};
use globalnews_storage::{DatabaseManager, Repository};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct IngestionPipeline {
    db: Arc<DatabaseManager>,
    backoff_policy: BackoffPolicy,
}

impl IngestionPipeline {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            backoff_policy: BackoffPolicy::default(),
        }
    }

    pub fn with_backoff_policy(db: Arc<DatabaseManager>, backoff_policy: BackoffPolicy) -> Self {
        Self { db, backoff_policy }
    }

    /// Selects the appropriate FeedAdapter for a given FeedFormat.
    pub fn get_adapter(format: FeedFormat) -> Box<dyn FeedAdapter> {
        match format {
            FeedFormat::Rss2 => Box::new(Rss2Adapter),
            FeedFormat::Atom => Box::new(AtomAdapter),
            FeedFormat::JsonStructured => Box::new(UsgsEarthquakeAdapter),
        }
    }

    /// Executes an ingestion run for an individual feed endpoint.
    pub async fn ingest_endpoint(
        &self,
        transport: &dyn HttpTransport,
        source: &Source,
        endpoint: &FeedEndpoint,
    ) -> Result<IngestionRun, IngestionError> {
        if !source.enabled {
            return Err(IngestionError::SourceDisabled);
        }
        if !endpoint.enabled {
            return Err(IngestionError::SourceDisabled);
        }

        let now = Utc::now();
        let now_str = now.to_rfc3339();

        // 1. Check Source Health and Backoff Window
        let health = self.db.with_reader(|conn| {
            Repository::get_source_health(conn, &source.id)
        })?.unwrap_or_else(|| SourceHealth::new(&source.id));

        if let Some(ref backoff_until) = health.backoff_until {
            if let Ok(backoff_dt) = chrono::DateTime::parse_from_rfc3339(backoff_until) {
                if now < backoff_dt {
                    info!(
                        "Source {} is in backoff until {}; skipping ingestion",
                        source.id, backoff_until
                    );
                    return Err(IngestionError::InBackoff(backoff_until.clone()));
                }
            }
        }

        // 2. Load Checkpoint for Conditional Request
        let checkpoint = self.db.with_reader(|conn| {
            Repository::get_feed_checkpoint(conn, &endpoint.id)
        })?.unwrap_or_default();

        let run_id = Uuid::new_v4().to_string();
        let mut run = IngestionRun::new(&run_id, &endpoint.id, &now_str);

        // 3. Build HTTP Request
        let mut req = HttpRequest::new(&endpoint.endpoint_url);
        req.etag = checkpoint.etag.clone();
        req.last_modified = checkpoint.last_modified.clone();

        info!(
            "Starting ingestion for endpoint {} ({}) [ETag: {:?}, Last-Modified: {:?}]",
            endpoint.id, endpoint.endpoint_url, req.etag, req.last_modified
        );

        // 4. Execute HTTP Transport
        let response_result = transport.execute(&req).await;

        let mut conn = self.db.get_writer()?;

        match response_result {
            // Case A: 304 Not Modified
            Ok(HttpResponse { status_code: 304, etag, last_modified, .. }) => {
                info!("Endpoint {} returned 304 Not Modified", endpoint.id);

                run.completed_at = Some(Utc::now().to_rfc3339());
                run.status = IngestionStatus::NotModified;
                run.http_status_code = Some(304);

                let updated_cp = FeedCheckpoint {
                    endpoint_id: endpoint.id.clone(),
                    etag: etag.or(checkpoint.etag),
                    last_modified: last_modified.or(checkpoint.last_modified),
                    last_retrieval_at: Some(Utc::now().to_rfc3339()),
                    last_status_code: Some(304),
                    last_successful_article_count: checkpoint.last_successful_article_count,
                    content_hash: checkpoint.content_hash,
                };

                let mut updated_health = health;
                updated_health.total_requests += 1;
                updated_health.successful_requests += 1;
                updated_health.consecutive_failures = 0;
                updated_health.backoff_until = None;
                updated_health.updated_at = Utc::now().to_rfc3339();

                Repository::upsert_feed_checkpoint(&conn, &updated_cp)?;
                Repository::record_ingestion_run(&conn, &run)?;
                Repository::update_source_health(&conn, &updated_health)?;

                Ok(run)
            }

            // Case B: Rate Limited (429)
            Err(IngestionError::RateLimited { retry_after_secs }) => {
                warn!("Endpoint {} was rate limited (429)", endpoint.id);

                let backoff_secs = self.backoff_policy.calculate_backoff_secs(
                    health.consecutive_failures + 1,
                    retry_after_secs,
                );
                let backoff_until = (now + chrono::Duration::seconds(backoff_secs as i64)).to_rfc3339();

                let mut updated_health = health;
                updated_health.total_requests += 1;
                updated_health.failed_requests += 1;
                updated_health.rate_limited_count += 1;
                updated_health.consecutive_failures += 1;
                updated_health.backoff_until = Some(backoff_until.clone());
                updated_health.last_error_message = Some(format!("HTTP 429 Rate Limited; backoff {}s", backoff_secs));
                updated_health.last_error_at = Some(now_str.clone());
                updated_health.updated_at = Utc::now().to_rfc3339();

                run.completed_at = Some(Utc::now().to_rfc3339());
                run.status = IngestionStatus::Failed;
                run.http_status_code = Some(429);
                run.error_message = Some("HTTP 429 Rate Limited".into());

                let err = RetrievalError {
                    id: Uuid::new_v4().to_string(),
                    run_id: Some(run.id.clone()),
                    endpoint_id: endpoint.id.clone(),
                    error_kind: ErrorKind::Http429,
                    details: format!("HTTP 429 Too Many Requests; backoff until {}", backoff_until),
                    occurred_at: now_str,
                };

                Repository::record_ingestion_run(&conn, &run)?;
                Repository::record_retrieval_error(&conn, &err)?;
                Repository::update_source_health(&conn, &updated_health)?;

                Err(IngestionError::RateLimited { retry_after_secs })
            }

            // Case C: Other Transport or HTTP Errors
            Err(e) => {
                error!("Endpoint {} retrieval failed: {}", endpoint.id, e);

                let backoff_secs = self.backoff_policy.calculate_backoff_secs(
                    health.consecutive_failures + 1,
                    None,
                );
                let backoff_until = (now + chrono::Duration::seconds(backoff_secs as i64)).to_rfc3339();

                let mut updated_health = health;
                updated_health.total_requests += 1;
                updated_health.failed_requests += 1;
                updated_health.consecutive_failures += 1;
                updated_health.backoff_until = Some(backoff_until);
                updated_health.last_error_message = Some(e.to_string());
                updated_health.last_error_at = Some(now_str.clone());
                updated_health.updated_at = Utc::now().to_rfc3339();

                run.completed_at = Some(Utc::now().to_rfc3339());
                run.status = IngestionStatus::Failed;
                run.error_message = Some(e.to_string());

                let err_kind = match e {
                    IngestionError::Timeout(_) => ErrorKind::NetworkTimeout,
                    IngestionError::PayloadTooLarge { .. } => ErrorKind::PayloadTooLarge,
                    IngestionError::MimeTypeMismatch { .. } => ErrorKind::MimeTypeMismatch,
                    IngestionError::SsrfBlocked(_) => ErrorKind::SsrfBlocked,
                    IngestionError::HttpStatus { status, .. } if status >= 500 => ErrorKind::Http5xx,
                    IngestionError::HttpStatus { .. } => ErrorKind::Http4xx,
                    _ => ErrorKind::Other,
                };

                let err = RetrievalError {
                    id: Uuid::new_v4().to_string(),
                    run_id: Some(run.id.clone()),
                    endpoint_id: endpoint.id.clone(),
                    error_kind: err_kind,
                    details: e.to_string(),
                    occurred_at: now_str,
                };

                Repository::record_ingestion_run(&conn, &run)?;
                Repository::record_retrieval_error(&conn, &err)?;
                Repository::update_source_health(&conn, &updated_health)?;

                Err(e)
            }

            // Case D: HTTP 200 OK — Parsing and Normalization
            Ok(resp) => {
                let adapter = Self::get_adapter(endpoint.feed_format);
                let raw_items = match adapter.parse(&resp.body, &endpoint.endpoint_url) {
                    Ok(items) => items,
                    Err(parse_err) => {
                        error!("Failed to parse feed payload for {}: {}", endpoint.id, parse_err);

                        run.completed_at = Some(Utc::now().to_rfc3339());
                        run.status = IngestionStatus::Failed;
                        run.http_status_code = Some(resp.status_code);
                        run.error_message = Some(parse_err.to_string());

                        let err = RetrievalError {
                            id: Uuid::new_v4().to_string(),
                            run_id: Some(run.id.clone()),
                            endpoint_id: endpoint.id.clone(),
                            error_kind: ErrorKind::XmlParseError,
                            details: parse_err.to_string(),
                            occurred_at: now_str,
                        };

                        Repository::record_ingestion_run(&conn, &run)?;
                        Repository::record_retrieval_error(&conn, &err)?;
                        return Err(parse_err);
                    }
                };

                let mut articles_new = 0;
                let mut articles_dup = 0;

                for raw_item in &raw_items {
                    match ArticleNormalizer::normalize(raw_item, source, Some(endpoint)) {
                        Ok(norm_art) => {
                            match Repository::insert_normalized_article_if_unique(&mut conn, &norm_art)? {
                                InsertResult::Inserted(_) => articles_new += 1,
                                InsertResult::Duplicate { .. } => articles_dup += 1,
                            }
                        }
                        Err(norm_err) => {
                            warn!("Normalization skipped an item in {}: {}", endpoint.id, norm_err);
                        }
                    }
                }

                run.completed_at = Some(Utc::now().to_rfc3339());
                run.status = IngestionStatus::Success;
                run.http_status_code = Some(resp.status_code);
                run.articles_fetched = raw_items.len();
                run.articles_new = articles_new;
                run.articles_duplicate = articles_dup;

                let updated_cp = FeedCheckpoint {
                    endpoint_id: endpoint.id.clone(),
                    etag: resp.etag,
                    last_modified: resp.last_modified,
                    last_retrieval_at: Some(Utc::now().to_rfc3339()),
                    last_status_code: Some(resp.status_code),
                    last_successful_article_count: raw_items.len() as u32,
                    content_hash: None,
                };

                let mut updated_health = health;
                updated_health.total_requests += 1;
                updated_health.successful_requests += 1;
                updated_health.consecutive_failures = 0;
                updated_health.backoff_until = None;
                updated_health.updated_at = Utc::now().to_rfc3339();

                Repository::upsert_feed_checkpoint(&conn, &updated_cp)?;
                Repository::record_ingestion_run(&conn, &run)?;
                Repository::update_source_health(&conn, &updated_health)?;

                info!(
                    "Ingestion complete for {}: {} fetched, {} new, {} duplicate",
                    endpoint.id, raw_items.len(), articles_new, articles_dup
                );

                Ok(run)
            }
        }
    }
}
