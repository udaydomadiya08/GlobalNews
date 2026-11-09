-- Migration 001: Initial Relational and Full-Text Search Schema

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL
);

-- Persistent Macro Events (Multi-month or Multi-year containers)
CREATE TABLE IF NOT EXISTS macro_events (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    category_code TEXT NOT NULL,
    summary TEXT NOT NULL,
    status TEXT NOT NULL,
    min_lat REAL NOT NULL,
    min_lon REAL NOT NULL,
    max_lat REAL NOT NULL,
    max_lon REAL NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Localized Incident Micro Events (Hotspots)
CREATE TABLE IF NOT EXISTS micro_events (
    id TEXT PRIMARY KEY,
    macro_event_id TEXT REFERENCES macro_events(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    category_code TEXT NOT NULL,
    importance_score REAL NOT NULL,
    confidence_score REAL NOT NULL,
    latitude REAL NOT NULL,
    longitude REAL NOT NULL,
    uncertainty_radius_km REAL NOT NULL DEFAULT 10.0,
    status TEXT NOT NULL,
    first_seen_at TEXT NOT NULL,
    last_updated_at TEXT NOT NULL,
    citation_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_micro_events_geo ON micro_events (latitude, longitude);
CREATE INDEX IF NOT EXISTS idx_micro_events_status ON micro_events (status);
CREATE INDEX IF NOT EXISTS idx_micro_events_updated ON micro_events (last_updated_at);
CREATE INDEX IF NOT EXISTS idx_micro_events_macro ON micro_events (macro_event_id);

-- Authorized Feed and Sensor Sources
CREATE TABLE IF NOT EXISTS sources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    tier INTEGER NOT NULL,
    feed_url TEXT NOT NULL UNIQUE,
    feed_type TEXT NOT NULL,
    reliability_score REAL NOT NULL,
    etag TEXT,
    last_modified TEXT,
    last_polled_at TEXT
);

-- Ingested Articles and Sensor Bulletins
CREATE TABLE IF NOT EXISTS articles (
    id TEXT PRIMARY KEY,
    micro_event_id TEXT REFERENCES micro_events(id) ON DELETE SET NULL,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    body_excerpt TEXT NOT NULL,
    language TEXT NOT NULL,
    source_id TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    content_hash TEXT NOT NULL UNIQUE,
    published_at TEXT NOT NULL,
    ingested_at TEXT NOT NULL,
    is_syndicated INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_articles_hash ON articles (content_hash);
CREATE INDEX IF NOT EXISTS idx_articles_micro ON articles (micro_event_id);
CREATE INDEX IF NOT EXISTS idx_articles_published ON articles (published_at);

-- Directed Semantic Event Relationships
CREATE TABLE IF NOT EXISTS event_relationships (
    source_event_id TEXT NOT NULL REFERENCES micro_events(id) ON DELETE CASCADE,
    target_event_id TEXT NOT NULL REFERENCES micro_events(id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL,
    description TEXT,
    PRIMARY KEY (source_event_id, target_event_id, relationship_type)
);

-- Chronological Event Timelines
CREATE TABLE IF NOT EXISTS event_timeline (
    id TEXT PRIMARY KEY,
    micro_event_id TEXT NOT NULL REFERENCES micro_events(id) ON DELETE CASCADE,
    timestamp TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    citation_url TEXT
);

CREATE INDEX IF NOT EXISTS idx_timeline_event ON event_timeline (micro_event_id, timestamp);

-- Evidence Corroboration Records
CREATE TABLE IF NOT EXISTS evidence_records (
    id TEXT PRIMARY KEY,
    micro_event_id TEXT NOT NULL REFERENCES micro_events(id) ON DELETE CASCADE,
    source_id TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    article_id TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    corroboration_weight REAL NOT NULL,
    UNIQUE (micro_event_id, article_id)
);

-- Full-Text Search Virtual Table for Micro Events (BM25 ranking)
CREATE VIRTUAL TABLE IF NOT EXISTS fts_events USING fts5(
    title,
    summary,
    content='micro_events',
    content_rowid='rowid'
);

-- Full-Text Search Virtual Table for Articles
CREATE VIRTUAL TABLE IF NOT EXISTS fts_articles USING fts5(
    title,
    body_excerpt,
    content='articles',
    content_rowid='rowid'
);
