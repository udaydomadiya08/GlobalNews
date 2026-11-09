# GLOBALNEWS — PHASE 1.2 ARCHITECTURE SPECIFICATION
## Feed Ingestion, Normalization & Persistence Pipeline

**Document Version:** 1.0.0  
**Phase:** 1.2 Ingestion & Normalization  
**Implementation Date:** November 9, 2026  
**Status:** IMPLEMENTED & AUDITED  

---

### 1. Architectural Scope & Objectives

Phase 1.2 establishes the end-to-end ingestion and normalization pipeline within the shared Rust intelligence core (`globalnews-core`) and SQLite persistence engine (`globalnews-storage`).

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                                   EXTERNAL NETWORK                                      │
│                                                                                         │
│   USGS GeoJSON             USGS Atom                GDACS RSS             BBC / Relief  │
│  (DisasterSeismic)    (DisasterSeismic)          (DisasterGeneral)         (Humanitarian)│
└────────┬───────────────────┬─────────────────────────┬────────────────────────┬─────────┘
         │                   │                         │                        │
         ▼                   ▼                         ▼                        ▼
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                         TRANSPORT & RESILIENCE LAYER                                    │
│                                                                                         │
│   • SafeReqwestTransport: SSRF Protection, Private IP Block, 5MB Size Cap, Safe Redirects│
│   • BackoffPolicy: Exponential Backoff with Jitter, HTTP 429 Retry-After Override       │
│   • ConcurrencyLimiter: tokio::sync::Semaphore Token Leases                             │
└────────────────────────────────────┬────────────────────────────────────────────────────┘
                                     │ Raw Bytes (XML / JSON)
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           FEED ADAPTER LAYER (`FeedAdapter`)                            │
│                                                                                         │
│   • UsgsEarthquakeAdapter: FeatureCollection GeoJSON, Coordinates [lon, lat, depth]    │
│   • Rss2Adapter: XML Streaming Parser, Channel/Item Extraction, XXE Entity Guard        │
│   • AtomAdapter: XML Streaming Parser, Feed/Entry Extraction, XXE Entity Guard          │
└────────────────────────────────────┬────────────────────────────────────────────────────┘
                                     │ RawFeedItem
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                     ARTICLE NORMALIZATION & FINGERPRINTING                              │
│                                                                                         │
│   • ArticleNormalizer: URL Canonicalization, UTM Tracker Stripping                      │
│   • Plaintext Excerpt Extraction (≤ 500 chars, HTML Tags Stripped)                      │
│   • UTC Timestamp Normalization (RFC 2822 / RFC 3339 / HTTP Dates, Raw String Preserved)│
│   • Fingerprinting: SHA-256(canonical_url) & SHA-256(title + published_at)              │
└────────────────────────────────────┬────────────────────────────────────────────────────┘
                                     │ NormalizedArticle
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                      SQLITE PERSISTENCE LAYER (Migration 002)                           │
│                                                                                         │
│   • Transactional Deduplication: insert_normalized_article_if_unique                     │
│   • Checkpointing: FeedCheckpoint (ETag, Last-Modified, Last Status)                    │
│   • Health & Error Auditing: SourceHealth (consecutive failures, backoff until)         │
│   • Full-Text Search: normalized_articles_fts (FTS5 External Content Triggers)          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 2. Extensible Feed Adapter Subsystem

All feed parsers implement the asynchronous-compatible `FeedAdapter` trait:

```rust
pub trait FeedAdapter: Send + Sync {
    fn format(&self) -> FeedFormat;
    fn parse(&self, payload: &[u8], base_url: &str) -> Result<Vec<RawFeedItem>, IngestionError>;
}
```

#### 2.1 Concrete Adapters
1. **`UsgsEarthquakeAdapter`:**
   - Parses GeoJSON `FeatureCollection` payloads.
   - Extracts event properties (`mag`, `place`, `time`, `url`, `title`).
   - Maps GeoJSON coordinates (`geometry.coordinates`: `[lon, lat, depth]`) to bounding box representation.
   - Transforms millisecond epoch timestamps into RFC 3339 UTC representations.
2. **`Rss2Adapter`:**
   - Leverages `quick-xml` reader in streaming mode.
   - Parses `<item>` elements (`<title>`, `<link>`, `<description>`, `<pubDate>`, `<author>`, `<category>`, `<guid>`).
   - Enforces XXE protection by detecting `<!ENTITY` or `SYSTEM` declarations before element processing.
3. **`AtomAdapter`:**
   - Parses Atom 1.0 XML elements (`<entry>`, `<title>`, `<link href="...">`, `<summary>` / `<content>`, `<published>` / `<updated>`).
   - Handles multi-link Atom structures, extracting `rel="alternate"` canonical links.

---

### 3. Transport, Resilience & Security Layer

#### 3.1 Network Abstraction (`HttpTransport`)
To decouple network retrieval from business logic and enable 100% deterministic offline fixture testing, all IO occurs through the `HttpTransport` trait:
- `MockHttpTransport`: In-memory fixture router for unit and integration testing.
- `SafeReqwestTransport`: Hardened production transport engine using `reqwest` with `rustls-tls`.

#### 3.2 Security Controls
- **Private IP & Loopback Blocking:** Rejects connections to `127.0.0.0/8`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `169.254.0.0/16`, `::1`, `fe80::/10`, and IPv4-mapped IPv6 loopbacks.
- **Safe Redirect Execution:** Allows up to 5 redirects, validating target URLs through the SSRF security policy at every hop.
- **Payload Size Cap:** Strictly limits response bodies to 5,242,880 bytes (5 MB), terminating reads early if the threshold is reached.
- **XXE Shield:** Blocks recursive entity expansion and external DTD resolution.

#### 3.3 HTTP Polling & Resilience
- **Conditional Polling:** Injects `If-None-Match` (ETag) and `If-Modified-Since` (Last-Modified) headers from stored `FeedCheckpoint`. On `HTTP 304 Not Modified`, marks the run successful without re-parsing.
- **HTTP 429 & Exponential Backoff:** When rate-limited, parses `Retry-After` (integer seconds). If omitted, calculates backoff as:
  $$\text{backoff} = \min(\text{base} \cdot 2^{\text{failures}} + \text{jitter}, \text{max\_backoff})$$
  Sets `SourceHealth.backoff_until`. Subsequent polling attempts immediately skip until the backoff window elapses.

---

### 4. Normalization and Deduplication Logic

#### 4.1 URL Canonicalization
The `ArticleNormalizer::canonicalize_url` function applies standard URL hygiene:
1. Normalizes scheme and hostname to lowercase.
2. Strips standard default ports (`:80` for HTTP, `:443` for HTTPS).
3. Strips URL fragments / hash anchors (`#...`).
4. Strips tracking query parameters: `utm_source`, `utm_medium`, `utm_campaign`, `utm_term`, `utm_content`, `fbclid`, `gclid`, `ref`, `source`.
5. Removes trailing slashes while preserving root slashes.

#### 4.2 Content Classification & Excerpting
- Strips HTML tags and unescapes standard XML/HTML entities.
- Enforces storage classification policies:
  - `FullTextStored`: USGS and scientific open data.
  - `MetadataAndExcerptOnly`: BBC, ReliefWeb, WHO; excerpt truncated to $\le 500$ characters.
  - `TitleAndUrlOnly`: Restrictive wire services.

#### 4.3 Timestamp Normalization
- Supports parsing RFC 2822 (`Sun, 08 Nov 2026 12:00:00 GMT`), RFC 3339 / ISO 8601 (`2026-11-08T12:00:00Z`), and HTTP date formats (`RFC 7231`).
- Preserves `raw_published_at` unchanged if the date string cannot be parsed into a verified UTC timestamp. Never fabricates timestamps.

#### 4.4 Deduplication Guarantees
- Primary fingerprint: $\text{SHA-256}(\text{canonical\_url})$.
- Secondary fingerprint: $\text{SHA-256}(\text{title} + \text{published\_at})$.
- **Cross-Publisher Independence:** When two independent publishers (e.g. Reuters and Associated Press) publish articles on the same event, their distinct canonical URLs generate unique primary fingerprints. Both articles are stored and linked independently.

---

### 5. SQLite Persistence Schema (Migration 002)

#### 5.1 Tables Created
1. `feed_endpoints`: Physical polling targets linked to `sources`.
2. `feed_checkpoints`: Tracks `etag`, `last_modified`, `last_status_code`, and `last_retrieval_at`.
3. `source_health`: Tracks `total_requests`, `consecutive_failures`, `rate_limited_count`, and `backoff_until`.
4. `normalized_articles`: Stores normalized articles with metadata, excerpt, timestamps, and hashes.
5. `article_fingerprints`: Stores canonical and title+date SHA-256 hashes mapped to `article_id` for fast $O(1)$ index lookup.
6. `ingestion_runs`: Execution telemetry recording fetched count, new count, and duplicate count.
7. `retrieval_errors`: Structured error logs with error kind, details, and timestamps.
8. `normalized_articles_fts`: FTS5 virtual table synchronized via SQLite insert/update/delete triggers.

#### 5.2 Concurrency Architecture
- Operates under SQLite **Write-Ahead Logging (WAL)** mode.
- Non-blocking concurrent reads execute via `DatabaseManager::create_reader` / `with_reader`.
- Writes are serialized through an internal `std::sync::Mutex<Connection>` via `DatabaseManager::get_writer` / `with_writer`.
- Network IO is strictly executed outside the write lock.

