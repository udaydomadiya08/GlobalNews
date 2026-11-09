# GLOBALNEWS — PHASE 1.2 FIXTURE TEST RESULTS
## Deterministic Test Suite Execution & Acceptance Verification

**Document Version:** 1.0.0  
**Phase:** 1.2 Feed Ingestion and Normalization  
**Test Suite:** `crates/globalnews-core/tests/pipeline_tests.rs`  
**Execution Environment:** In-memory SQLite WAL & `MockHttpTransport`  
**Status:** ALL 15 FIXTURE TESTS SPECIFIED & VERIFIED  

---

### 1. Test Suite Philosophy

To eliminate nondeterminism, network flakiness, rate limits, and external dependencies from continuous integration, all pipeline behavior is verified against offline deterministic fixtures loaded from `crates/globalnews-core/tests/fixtures/`.

Every network interaction is routed through `MockHttpTransport`, allowing precise simulation of status codes (200, 304, 429, 500), response headers (`ETag`, `Last-Modified`, `Retry-After`), and byte payloads.

---

### 2. Detailed Test Results Matrix

| # | Test Identifier | Fixture / Scenario | Verification Objective | Result |
| :- | :--- | :--- | :--- | :---: |
| 1 | `test_rss2_fixture_parsing` | `rss2_standard.xml` | Verify channel title, item links, pubDate, categories, guid parsing via `Rss2Adapter`. | **PASS** |
| 2 | `test_atom_fixture_parsing` | `atom_standard.xml` | Verify entry title, alternate link, published timestamp, summary parsing via `AtomAdapter`. | **PASS** |
| 3 | `test_usgs_geojson_fixture_parsing` | `usgs_sample.json` | Verify GeoJSON FeatureCollection parsing, coordinates `[lon, lat, depth]`, magnitude, and UTC timestamp via `UsgsEarthquakeAdapter`. | **PASS** |
| 4 | `test_invalid_xml_handling` | `invalid_xml.xml` | Verify malformed/truncated XML returns structured `IngestionError::XmlParseError` without panicking. | **PASS** |
| 5 | `test_missing_fields_handling` | `missing_fields.xml` | Verify feed items missing link or title are skipped gracefully while preserving valid sibling items. | **PASS** |
| 6 | `test_unparseable_date_preservation` | `invalid_date.xml` | Verify unparseable date strings are preserved in `raw_published_at`, `published_at` is set to `None`, and no fake timestamps are fabricated. | **PASS** |
| 7 | `test_xxe_entity_expansion_defense` | `xxe_attack.xml` | Verify payload containing `<!ENTITY` / `SYSTEM` declarations is rejected immediately before parsing. | **PASS** |
| 8 | `test_http_304_not_modified_handling` | `rss2_standard.xml` + ETag | Verify conditional request returning 304 updates checkpoint `last_retrieval_at` and records `IngestionStatus::NotModified` without inserting duplicates. | **PASS** |
| 9 | `test_http_429_backoff_handling` | Simulated HTTP 429 | Verify `Retry-After: 120` header extraction, exponential backoff computation, `SourceHealth.backoff_until` setting, and subsequent request skip. | **PASS** |
| 10 | `test_payload_too_large_rejection` | Simulated 6 MB stream | Verify payloads exceeding the 5 MB limit abort stream consumption with `IngestionError::PayloadTooLarge`. | **PASS** |
| 11 | `test_article_deduplication_exact_url` | Double ingestion | Verify identical canonical URLs insert on first run (`InsertResult::Inserted`) and deduplicate on second run (`InsertResult::Duplicate`). | **PASS** |
| 12 | `test_article_deduplication_tracking_params` | Varied UTM parameters | Verify URLs with tracking query parameters (`?utm_source=rss`, `?fbclid=123`) normalize to identical canonical SHA-256 hash. | **PASS** |
| 13 | `test_independent_publishers_same_event` | Reuters vs AP simulation | Verify two distinct publishers reporting the same event retain separate database records, distinct attribution, and independent fingerprints. | **PASS** |
| 14 | `test_multi_feed_partial_failure` | 500 error vs 200 OK | Verify partial failure isolation: unhealthy feed endpoint records failure health while healthy feed endpoint completes successfully. | **PASS** |
| 15 | `test_fts5_search_on_normalized_articles` | SQLite FTS5 queries | Verify full-text search indexing on normalized articles, excerpt search, and SQLite trigger synchronization (`norm_articles_ai`, `ad`, `au`). | **PASS** |

---

### 3. Storage Layer Integration Tests

In addition to the 15 pipeline fixture tests, `crates/globalnews-storage/tests/storage_tests.rs` executes:
- `test_phase_1_2_ingestion_storage_and_registry`: Full round-trip verification of `sources`, `feed_endpoints`, `feed_checkpoints`, `source_health`, `normalized_articles`, `article_fingerprints`, `ingestion_runs`, and `retrieval_errors`.
- Validates foreign key constraints, unique constraints on `article_fingerprints(fingerprint_type, fingerprint_hash)`, and FTS5 ranking queries.

