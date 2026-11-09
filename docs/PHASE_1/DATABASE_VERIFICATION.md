# GLOBALNEWS — PHASE 1.1.1: DATABASE IMPLEMENTATION VERIFICATION

**Date:** 2026-11-09  
**Status:** Architecture Audited & Schema Corrected  
**Module:** `crates/globalnews-storage/`  
**Storage Engine:** Embedded SQLite 3.45+ via `rusqlite` (WAL Mode & FTS5)

---

## 1. Concurrency Model & Connection Architecture

The Phase 1.1 documentation contained an architectural discrepancy claiming that `r2d2` connection pooling was utilized. Inspection of `crates/globalnews-storage/src/database.rs` revealed the actual implementation:

```rust
pub struct DatabaseManager {
    config: DatabaseConfig,
    // Dedicated mutex-guarded write connection ensuring single-writer serialization
    write_conn: Mutex<Connection>,
}
```

### 1.1 Single-Writer Serialization Invariant
SQLite in WAL mode permits unlimited concurrent readers but strictly **one active writer**. Attempting to write concurrently from multiple threads produces `SQLITE_BUSY`.
- The `DatabaseManager` wraps the write connection in a `Mutex<Connection>`.
- Any write operation is executed via:
  ```rust
  pub fn with_writer<F, R>(&self, f: F) -> Result<R, StorageError>
  where
      F: FnOnce(&mut Connection) -> Result<R, StorageError>,
  ```
- This guarantees that concurrent write requests are serialized in-process before reaching the SQLite C engine, eliminating `SQLITE_BUSY` contention.

### 1.2 Lockless Concurrent Readers
For read operations (search, spatial event queries, article lookups):
- `DatabaseManager::create_reader()` opens read-only connections (`SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX`).
- In SQLite WAL mode, readers do not block writers, and writers do not block readers.
- During Phase 1.1.1, `DatabaseManager::with_reader()` was added and all read commands in `globalnews-app` were migrated away from the writer lock.

### 1.3 In-Memory Shared Cache Correction
Previously, `create_reader()` on an in-memory database (`path: None`) invoked `Connection::open_in_memory()`, opening an isolated database that could not read committed data.
- **Correction:** When `config.path` is `None`, both writer and readers now connect to a shared in-memory URI:
  ```rust
  const SHARED_IN_MEMORY_URI: &str = "file:globalnews_mem?mode=memory&cache=shared";
  ```
  with `SQLITE_OPEN_URI`, allowing concurrent read and write operations across in-memory connections.

---

## 2. WAL Configuration & Operational Pragmas

Every connection created by `DatabaseManager` automatically executes the following pragmas:

```sql
PRAGMA busy_timeout = 5000;
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
```

| Pragma | Setting | Rationale |
|---|---|---|
| `busy_timeout` | `5000` (ms) | Prevents immediate failure during transient table locks; waits up to 5s. |
| `foreign_keys` | `ON` | Enforces relational integrity across macro/micro events, sources, and evidence. |
| `journal_mode` | `WAL` | Enables concurrent readers and append-only write performance. |
| `synchronous` | `NORMAL` | Sufficient for WAL mode; avoids full disk flush on every commit while guaranteeing consistency against application crashes. |

---

## 3. Schema Migrations & Transactional Rollback

The migration engine (`crates/globalnews-storage/src/migrations/mod.rs`) executes numbered SQL scripts:
- Migration versioning is tracked in table `schema_migrations (version INTEGER PRIMARY KEY, name TEXT, applied_at TEXT)`.
- **Transactional Rollback:** Each migration is executed inside a transaction:
  ```rust
  let tx = conn.transaction()?;
  tx.execute_batch(migration.sql)?;
  tx.execute("INSERT INTO schema_migrations ...")?;
  tx.commit()?;
  ```
- If any statement in `001_initial_schema.sql` fails, the transaction is automatically rolled back on drop, preventing partial or corrupt schema states.

---

## 4. Elimination of Data Loss Placeholders in `Repository`

### Defect Identified
In Phase 1.1, `micro_events` table omitted `toponym_name`, `country_code`, and scoring sub-components. `Repository::get_micro_event()` was forced to inject placeholder values:
```rust
// Previous placeholder code in get_micro_event:
toponym_name: "Resolved Coordinate".into(),
country_code: "XX".into(),
geographic_reach: 5.0,
```

### Schema Correction
In Phase 1.1.1, `001_initial_schema.sql` was updated to persist all 27 entity attributes:
- `domain_severity`, `geographic_reach`, `physical_impact`, `volume_dampener`
- `independent_source_count`, `jurisdictional_diversity`, `has_sensor_validation`, `is_disputed`
- `altitude_m`, `toponym_name`, `geonames_id`, `country_code`, `admin1_code`

`Repository::insert_micro_event` and `Repository::get_micro_event` were updated to read and write all 27 fields. Full round-trip fidelity is now guaranteed.

---

## 5. FTS5 Virtual Tables & Automatic Triggers

### Defect Identified
Previously, `fts_events` and `fts_articles` lacked synchronization triggers. Manual FTS inserts in `Repository` resulted in rowid collisions upon record updates.

### Schema Correction
Added 6 transactional triggers in `001_initial_schema.sql`:
```sql
-- micro_events triggers
CREATE TRIGGER IF NOT EXISTS micro_events_ai AFTER INSERT ON micro_events BEGIN
    INSERT INTO fts_events(rowid, title, summary) VALUES (new.rowid, new.title, new.summary);
END;
CREATE TRIGGER IF NOT EXISTS micro_events_ad AFTER DELETE ON micro_events BEGIN
    INSERT INTO fts_events(fts_events, rowid, title, summary) VALUES('delete', old.rowid, old.title, old.summary);
END;
CREATE TRIGGER IF NOT EXISTS micro_events_au AFTER UPDATE ON micro_events BEGIN
    INSERT INTO fts_events(fts_events, rowid, title, summary) VALUES('delete', old.rowid, old.title, old.summary);
    INSERT INTO fts_events(rowid, title, summary) VALUES (new.rowid, new.title, new.summary);
END;

-- articles triggers
CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
    INSERT INTO fts_articles(rowid, title, body_excerpt) VALUES (new.rowid, new.title, new.body_excerpt);
END;
CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
    INSERT INTO fts_articles(fts_articles, rowid, title, body_excerpt) VALUES('delete', old.rowid, old.title, old.body_excerpt);
END;
CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
    INSERT INTO fts_articles(fts_articles, rowid, title, body_excerpt) VALUES('delete', old.rowid, old.title, old.body_excerpt);
    INSERT INTO fts_articles(rowid, title, body_excerpt) VALUES (new.rowid, new.title, new.body_excerpt);
END;
```

**Verification:** FTS5 BM25 ranked queries now operate with zero manual synchronization code in Rust.
