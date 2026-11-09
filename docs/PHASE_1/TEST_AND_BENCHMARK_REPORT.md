# GLOBALNEWS — PHASE 1.1: TEST AND BENCHMARK SPECIFICATION REPORT

**Date:** 2026-11-09  
**Status:** Foundation Phase 1.1 Test Suite & Performance Invariants  
**Target Architecture:** Apple Silicon Darwin aarch64 (M1), Multiplatform Cross-Target Verification

---

## 1. Test Architecture & Coverage Strategy

The test strategy for Phase 1.1 follows the strict layered separation of the GLOBALNEWS workspace:

```
crates/
├── globalnews-contracts/tests/contract_tests.rs
│   ├── LocationValidation (WGS84 lat/long boundaries)
│   ├── ScoringRangeValidation (0.0 - 1.0 clamped invariant)
│   ├── ArticleSerializationRoundtrip (JSON / Serde)
│   ├── MacroMicroEventHierarchies (1:N parent-child relations)
│   └── SyncEnvelopeConsistency (Vector clocks & checksums)
│
├── globalnews-storage/tests/storage_tests.rs
│   ├── MigrationRunner (Idempotent 001_initial_schema.sql)
│   ├── WALPragmasVerification (WAL mode, busy_timeout = 5000)
│   ├── ArticleCrudAndUniqueness (Duplicate URL collision handling)
│   ├── SourceRegistryAndTrust (Trust tiers & language tracking)
│   ├── EventPersistenceAndFiltering (Spatial bounding box & time range)
│   └── Fts5Bm25Search (Lexical full-text queries & ranked relevance)
│
├── globalnews-platform/src/lib.rs (Unit tests)
│   ├── DesktopAdapterPaths (Data/cache/config directory resolution)
│   ├── MobileAdapterConstraints (Background refresh token handling)
│   └── PlatformCapabilities (Filesystem, notifications, network)
│
└── globalnews-core/tests/core_tests.rs
    ├── CoreStateInitialization (In-memory & disk persistence)
    ├── GracefulShutdownTrigger (Atomic cancellation flags)
    └── BaselineBenchmarkRunner (Contracts, storage, FTS5 throughput)
```

---

## 2. Test Suite Specification & Verification Criteria

### 2.1 Contract Verification (`globalnews-contracts`)

| Test Name | Target Invariant | Assertion Criteria |
|---|---|---|
| `test_location_validation` | WGS84 coordinates | Reject latitude < -90 or > 90; reject longitude < -180 or > 180. Pass within bounds. |
| `test_scoring_validation` | Clamped normalized floats | Reject negative or > 1.0 confidence/importance scores. Reject NaN and Infinity. |
| `test_article_roundtrip` | Serialization fidelity | Serialized JSON must match deserialized struct across all fields (UUID, URL, language, entities). |
| `test_event_hierarchy` | Macro/Micro linkage | MicroEvent `macro_event_id` must resolve; MicroEvent coordinates must match bounding polygon. |
| `test_sync_envelope` | Offline delta replication | Client batch payload must validate checksum, schema version (`v1.0.0`), and vector clock. |

### 2.2 Storage & Persistence Verification (`globalnews-storage`)

| Test Name | Target Invariant | Assertion Criteria |
|---|---|---|
| `test_migration_runner` | Forward migration | Apply `001_initial_schema.sql`, record version 1 in `schema_migrations`, re-run without error. |
| `test_wal_mode` | Concurrent read/write safety | `PRAGMA journal_mode` must evaluate to `wal`; `foreign_keys` must evaluate to `ON`. |
| `test_article_insert_and_find` | Relational integrity | Insert `ArticleRecord`, query by ID and URL, verify timestamp and metadata preservation. |
| `test_source_crud` | Feed catalog management | Insert source, update trust tier, query active sources by language code. |
| `test_event_spatial_filter` | Geographic indexing | Query events within latitude `[20.0, 50.0]` and longitude `[-10.0, 40.0]`; reject outliers. |
| `test_fts5_bm25_search` | Ranked search retrieval | Insert articles mentioning "earthquake" and "economic summit"; verify BM25 query returns ranked hits. |

### 2.3 Platform Boundary Verification (`globalnews-platform`)

| Test Name | Target Invariant | Assertion Criteria |
|---|---|---|
| `test_desktop_app_dirs` | Operating system paths | Resolves standard user app data folder: `~/Library/Application Support/globalnews` on macOS. |
| `test_platform_capabilities` | Native capability query | Desktop reports `supports_continuous_background: true`; mobile reports `false`. |

### 2.4 Core Integration & Benchmark Harness (`globalnews-core`)

| Test Name | Target Invariant | Assertion Criteria |
|---|---|---|
| `test_core_state_lifecycle` | Initialization & teardown | Create `CoreState`, verify storage pool readiness, trigger `shutdown()` and observe exit. |
| `test_baseline_benchmarks` | Performance budgets | Run `BenchmarkRunner::run_baseline()` and assert all metrics fall within memory/latency budgets. |

---

## 3. Baseline Performance Targets & Budgets

The following performance thresholds are hardcoded into `globalnews-core/src/benchmarks.rs` and enforced during testing:

| Metric | Target Budget | Target Rationale |
|---|---|---|
| **Contract Serialization** | **< 0.05 ms** (< 50 µs) | Zero-copy Serde deserialization must not hitch UI event loops. |
| **Contract Validation** | **< 0.02 ms** (< 20 µs) | Coordinate boundary and score clamping checks must be instantaneous. |
| **Storage Batch Write** | **< 5.00 ms** per 50 records | WAL mode transactional inserts must sustain 1,000 articles/sec ingestion. |
| **FTS5 BM25 Query** | **< 2.50 ms** across 10k index | Full-text prefix and keyword search must render in single-digit ms for 60 FPS search UX. |
| **In-Memory Core State** | **< 25 MB** baseline RAM | Portable core must leave >90% of mobile RAM quota (150 MB Jetsam limit) for visualizer. |

---

## 4. Test Execution Command Reference

```bash
# Set path to include Rust binaries
export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"

# Run all workspace unit and integration tests
cargo test --workspace --verbose

# Run contracts tests specifically
cargo test -p globalnews-contracts --test contract_tests

# Run storage and FTS5 search integration tests
cargo test -p globalnews-storage --test storage_tests

# Run core state and baseline benchmark tests
cargo test -p globalnews-core --test core_tests -- --nocapture
```
