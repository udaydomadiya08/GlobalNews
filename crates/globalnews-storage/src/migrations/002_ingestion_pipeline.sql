-- ===========================================================================
-- GLOBALNEWS PERSISTENCE ENGINE: MIGRATION 002
-- FEED INGESTION AND NORMALIZATION PIPELINE EXTENSION
-- ===========================================================================

-- 1. Extend authorized sources table with registry metadata
ALTER TABLE sources ADD COLUMN canonical_domain TEXT NOT NULL DEFAULT '';
ALTER TABLE sources ADD COLUMN category TEXT NOT NULL DEFAULT 'general_news';
ALTER TABLE sources ADD COLUMN supported_languages TEXT NOT NULL DEFAULT '["en"]';
ALTER TABLE sources ADD COLUMN geographic_coverage TEXT DEFAULT 'GLOBAL';
ALTER TABLE sources ADD COLUMN attribution_requirements TEXT NOT NULL DEFAULT '';
ALTER TABLE sources ADD COLUMN licence_policy_url TEXT;
ALTER TABLE sources ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1;
ALTER TABLE sources ADD COLUMN created_at TEXT NOT NULL DEFAULT '2026-01-01T00:00:00Z';
ALTER TABLE sources ADD COLUMN updated_at TEXT NOT NULL DEFAULT '2026-01-01T00:00:00Z';

-- 2. Feed Endpoints (supports multiple feeds per publisher source)
CREATE TABLE IF NOT EXISTS feed_endpoints (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    endpoint_url TEXT NOT NULL UNIQUE,
    feed_format TEXT NOT NULL,
    retrieval_method TEXT NOT NULL,
    polling_interval_secs INTEGER NOT NULL DEFAULT 1800,
    rate_limit_per_minute INTEGER,
    enabled INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 10,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_feed_endpoints_source ON feed_endpoints (source_id);
CREATE INDEX IF NOT EXISTS idx_feed_endpoints_enabled ON feed_endpoints (enabled);

-- 3. Feed Checkpoints (HTTP conditional caching and resumption)
CREATE TABLE IF NOT EXISTS feed_checkpoints (
    endpoint_id TEXT PRIMARY KEY REFERENCES feed_endpoints(id) ON DELETE CASCADE,
    etag TEXT,
    last_modified TEXT,
    last_retrieval_at TEXT,
    last_status_code INTEGER,
    last_successful_article_count INTEGER DEFAULT 0,
    content_hash TEXT
);

-- 4. Source Health & Operational Resilience
CREATE TABLE IF NOT EXISTS source_health (
    source_id TEXT PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    total_requests INTEGER NOT NULL DEFAULT 0,
    successful_requests INTEGER NOT NULL DEFAULT 0,
    failed_requests INTEGER NOT NULL DEFAULT 0,
    rate_limited_count INTEGER NOT NULL DEFAULT 0,
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    backoff_until TEXT,
    last_error_message TEXT,
    last_error_at TEXT,
    reliability_score REAL NOT NULL DEFAULT 1.0,
    updated_at TEXT NOT NULL
);

-- 5. Canonical Normalized Articles
CREATE TABLE IF NOT EXISTS normalized_articles (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    endpoint_id TEXT REFERENCES feed_endpoints(id) ON DELETE SET NULL,
    canonical_url TEXT NOT NULL,
    original_url TEXT NOT NULL,
    title TEXT NOT NULL,
    description_excerpt TEXT,
    content_classification TEXT NOT NULL DEFAULT 'metadata_only',
    published_at TEXT,
    raw_published_at TEXT,
    retrieved_at TEXT NOT NULL,
    updated_at TEXT,
    language TEXT,
    authors TEXT NOT NULL DEFAULT '[]',
    feed_categories TEXT NOT NULL DEFAULT '[]',
    source_provenance TEXT NOT NULL,
    is_syndicated INTEGER NOT NULL DEFAULT 0,
    canonical_url_hash TEXT NOT NULL,
    title_date_hash TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_norm_articles_canon_hash ON normalized_articles (canonical_url_hash);
CREATE INDEX IF NOT EXISTS idx_norm_articles_title_date ON normalized_articles (title_date_hash);
CREATE INDEX IF NOT EXISTS idx_norm_articles_source ON normalized_articles (source_id);
CREATE INDEX IF NOT EXISTS idx_norm_articles_endpoint ON normalized_articles (endpoint_id);
CREATE INDEX IF NOT EXISTS idx_norm_articles_published ON normalized_articles (published_at);
CREATE INDEX IF NOT EXISTS idx_norm_articles_retrieved ON normalized_articles (retrieved_at);

-- 6. Deduplication Fingerprints
CREATE TABLE IF NOT EXISTS article_fingerprints (
    fingerprint_hash TEXT PRIMARY KEY,
    hash_type TEXT NOT NULL,
    article_id TEXT NOT NULL REFERENCES normalized_articles(id) ON DELETE CASCADE,
    source_id TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    first_seen_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_article_fingerprints_article ON article_fingerprints (article_id);
CREATE INDEX IF NOT EXISTS idx_article_fingerprints_source ON article_fingerprints (source_id);

-- 7. Ingestion Run Audit Logs
CREATE TABLE IF NOT EXISTS ingestion_runs (
    id TEXT PRIMARY KEY,
    endpoint_id TEXT NOT NULL REFERENCES feed_endpoints(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL,
    http_status_code INTEGER,
    articles_fetched INTEGER NOT NULL DEFAULT 0,
    articles_new INTEGER NOT NULL DEFAULT 0,
    articles_duplicate INTEGER NOT NULL DEFAULT 0,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_ingestion_runs_endpoint ON ingestion_runs (endpoint_id);
CREATE INDEX IF NOT EXISTS idx_ingestion_runs_started ON ingestion_runs (started_at);

-- 8. Retrieval Error Tracking
CREATE TABLE IF NOT EXISTS retrieval_errors (
    id TEXT PRIMARY KEY,
    run_id TEXT REFERENCES ingestion_runs(id) ON DELETE SET NULL,
    endpoint_id TEXT NOT NULL REFERENCES feed_endpoints(id) ON DELETE CASCADE,
    error_kind TEXT NOT NULL,
    details TEXT NOT NULL,
    occurred_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_retrieval_errors_endpoint ON retrieval_errors (endpoint_id);

-- 9. FTS5 Virtual Table for Normalized Articles
CREATE VIRTUAL TABLE IF NOT EXISTS normalized_articles_fts USING fts5(
    id UNINDEXED,
    title,
    description_excerpt,
    source_provenance,
    content='normalized_articles',
    content_rowid='rowid'
);

-- Automated FTS5 Triggers for Synchronization
CREATE TRIGGER IF NOT EXISTS norm_articles_ai AFTER INSERT ON normalized_articles BEGIN
    INSERT INTO normalized_articles_fts(rowid, id, title, description_excerpt, source_provenance)
    VALUES (new.rowid, new.id, new.title, new.description_excerpt, new.source_provenance);
END;

CREATE TRIGGER IF NOT EXISTS norm_articles_ad AFTER DELETE ON normalized_articles BEGIN
    INSERT INTO normalized_articles_fts(normalized_articles_fts, rowid, id, title, description_excerpt, source_provenance)
    VALUES('delete', old.rowid, old.id, old.title, old.description_excerpt, old.source_provenance);
END;

CREATE TRIGGER IF NOT EXISTS norm_articles_au AFTER UPDATE ON normalized_articles BEGIN
    INSERT INTO normalized_articles_fts(normalized_articles_fts, rowid, id, title, description_excerpt, source_provenance)
    VALUES('delete', old.rowid, old.id, old.title, old.description_excerpt, old.source_provenance);
    INSERT INTO normalized_articles_fts(rowid, id, title, description_excerpt, source_provenance)
    VALUES (new.rowid, new.id, new.title, new.description_excerpt, new.source_provenance);
END;
