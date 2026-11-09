# GLOBALNEWS — Phase 1.1 Database Foundation & SQLite Persistence

**Document ID:** `GN-DOC-P11-006`  
**Phase:** Phase 1.1 (Portable Rust Core Foundation & Cross-Platform Scaffolding)  
**Status:** Approved Technical Architecture Specification  
**Date:** 2026-11-09  

---

## 1. Portable SQLite Persistence Strategy

GLOBALNEWS requires an embedded database engine that delivers identical SQL dialect behavior, full-text search algorithms, transactional semantics, and binary reliability across Windows, macOS, Linux, Android, and iOS.

Rather than relying on the host operating system's system SQLite library (which varies wildly from SQLite 3.28 on older Android versions to SQLite 3.43 on macOS), `globalnews-storage` utilizes **`rusqlite` with the `bundled` feature**:
* **Bundled C Source:** Compiles SQLite 3.45+ directly into the native binary using the target platform's C compiler.
* **Deterministic Configuration:** FTS5 (Full-Text Search 5), WAL (Write-Ahead Logging), Foreign Key cascades, and math functions are guaranteed to exist on every target device.
* **Zero System Linkage Errors:** Avoids dynamic library resolution failures on Android and iOS.

---

## 2. Multi-Platform SQLite Compatibility Matrix

```
+-------------------------------------------------------------------------------------------------------------------------+
|                                    SQLITE COMPATIBILITY & PRAGMA SUPPORT                                                |
+----------------------+-----------------------------+-----------------------+---------------------+----------------------+
| Platform             | SQLite Compilation Model    | WAL Mode Support      | FTS5 Support        | Foreign Keys         |
+----------------------+-----------------------------+-----------------------+---------------------+----------------------+
| **Windows Desktop**  | Bundled via MSVC / Clang    | Supported             | Supported           | Enforced (`ON`)      |
| **macOS Desktop**    | Bundled via Apple Clang     | Supported             | Supported           | Enforced (`ON`)      |
| **Linux Desktop**    | Bundled via GCC / Clang     | Supported             | Supported           | Enforced (`ON`)      |
| **Android Mobile**   | Bundled via Android NDK     | Supported             | Supported           | Enforced (`ON`)      |
| **iOS Mobile**       | Bundled via Xcode Clang     | Supported             | Supported           | Enforced (`ON`)      |
+----------------------+-----------------------------+-----------------------+---------------------+----------------------+
```

---

## 3. Pragma Configuration & Concurrency Architecture

Every database connection established by `DatabaseManager` runs four mandatory pragmas:

```sql
PRAGMA busy_timeout = 5000;
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
```

### 3.1 Why Single-Writer Serialization Is Mandatory
Under SQLite WAL mode:
* Multiple threads can read concurrently without locking each other or blocking the writer.
* **However, SQLite permits only ONE active write transaction at any instant.**
* If two asynchronous background ingestors attempt to write simultaneously, one will block and eventually throw `sqlite3.OperationalError: database is locked` if the busy timeout expires.

`globalnews-storage` eliminates write contention by design:
* **The Single-Writer Pattern:** All write operations are routed through `DatabaseManager::with_writer(|conn| { ... })`, which is guarded by an internal `Mutex<Connection>`.
* Writes are queued and serialized in memory before hitting SQLite, guaranteeing zero lock collisions.

---

## 4. Migration Engine & Version Tracking

Database evolution is managed deterministically through the `MigrationRunner`:
1. Checks the `schema_migrations` table:
   ```sql
   CREATE TABLE IF NOT EXISTS schema_migrations (
       version INTEGER PRIMARY KEY,
       name TEXT NOT NULL,
       applied_at TEXT NOT NULL
   );
   ```
2. Compares the current version (`PRAGMA user_version` or `MAX(version)`) against the statically embedded migration array `MIGRATIONS`.
3. Applies pending migrations inside atomic transactions (`tx.execute_batch(...)`).

### Initial Migration (Version 1: `001_initial_schema.sql`)
Creates the initial schema:
* `macro_events`: Multi-year megastories with bounding boxes.
* `micro_events`: Localized incident hotspots with coordinates and importance scores.
* `sources`: Authorized feed metadata, tiers, and HTTP caching headers (`etag`, `last_modified`).
* `articles`: Normalized news documents with unique SHA-256 `content_hash`.
* `event_relationships`: Directed edges (`CAUSES`, `DISRUPTS`, `ESCALATES`, etc.).
* `event_timeline`: Chronological progression entries.
* `evidence_records`: Cross-corroboration links.
* `fts_events` & `fts_articles`: FTS5 virtual tables with BM25 ranking.

---

## 5. Synchronization Policy: No Raw Database Copying

A common failure mode in local-first apps is copying the raw `.sqlite` file between devices (e.g. from Desktop to Phone).
* **The Hazard:** Copying an active WAL-mode SQLite database while processes are running corrupts `-wal` and `-shm` shared memory files, causes schema version conflicts, and creates merge collisions.
* **The Policy:** **Live SQLite database files are strictly local to each device.** Cross-device synchronization is performed exclusively through versioned `SyncEnvelope` contracts exchanging incremental entity records.
