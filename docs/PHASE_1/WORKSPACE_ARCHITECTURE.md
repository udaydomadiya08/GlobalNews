# GLOBALNEWS — Phase 1.1 Workspace Architecture & Crate Boundaries

**Document ID:** `GN-DOC-P11-002`  
**Phase:** Phase 1.1 (Portable Rust Core Foundation & Cross-Platform Scaffolding)  
**Status:** Approved Technical Baseline  
**Date:** 2026-11-09  

---

## 1. Architectural Philosophy: Purpose-Driven Modularity

A critical hazard in modern Rust systems is **gratuitous crate fragmentation**—splitting code into dozens of tiny single-file crates that introduce circular dependency hazards, bloat compilation times, and obscure data ownership.

GLOBALNEWS enforces strict, principled crate boundaries. Every crate in the workspace serves a non-negotiable architectural purpose:

```
+===================================================================================================+
|                                  GLOBALNEWS WORKSPACE CRATES                                      |
+===================================================================================================+
|                                                                                                   |
|  [ globalnews-app ] (Tauri 2.0 Application Shell)                                                 |
|  - In-process native IPC command handlers (get_health, search_events, get_micro_event)            |
|  - Capability-based permissions (capabilities/default.json)                                      |
|  - Platform packaging: Windows EXE, macOS DMG, Linux AppImage, Android APK, iOS IPA               |
|                                                |                                                  |
|                        +-----------------------+-----------------------+                          |
|                        |                                               |                          |
|                        v                                               v                          |
|  [ globalnews-platform ]                              [ globalnews-core ]                         |
|  - PlatformAdapter trait                              - EngineState orchestration                |
|  - DesktopAdapter (Paths, Tray, Notify)               - Structured logging & config              |
|  - MobileAdapter (Android/iOS JNI/Swift FFI)          - Cancellation & benchmarks                |
|                        |                                               |                          |
|                        +-----------------------+-----------------------+                          |
|                                                |                                                  |
|                                                v                                                  |
|  [ globalnews-storage ]                                                                           |
|  - rusqlite bundled SQLite 3.45+ engine (WAL mode, FTS5 BM25 search, foreign keys)                |
|  - MigrationRunner (tracks schema versions in schema_migrations)                                  |
|  - Single-writer mutex serialization & concurrent read pools                                      |
|                                                |                                                  |
|                                                v                                                  |
|  [ globalnews-contracts ]                                                                         |
|  - 100% pure, platform-agnostic, zero-dependency data structures                                  |
|  - MicroEvent, MacroEvent, GeographicLocation, ImportanceScore, ConfidenceScore, SyncEnvelope     |
|  - Strict validation, serde serialization, RFC 3339 normalization, schema version "1.0.0"         |
|                                                                                                   |
+===================================================================================================+
```

---

## 2. In-Depth Crate Specifications

### 2.1 `globalnews-contracts` (The Semantic Backbone)
* **Role:** Serves as the immutable, universal type vocabulary across all platforms and layers.
* **Dependencies:** `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`.
* **Prohibited Dependencies:** Zero database libraries (`rusqlite`), zero networking libraries (`reqwest`), zero UI or operating-system-specific crates.
* **Invariants:**
  * Must compile on any platform that supports Rust (Tier 1, Tier 2, and Tier 3 targets, including WebAssembly).
  * Every struct implements strict validation (`validate(&self) -> Result<(), ContractError>`).
  * All coordinates are bound to valid WGS84 ranges: $\text{lat} \in [-90.0, 90.0]$, $\text{lon} \in [-180.0, 180.0]$.
  * All scores are mathematically bounded: $\text{Importance} \in [1.0, 10.0]$, $\text{Confidence} \in [0.1, 1.0]$.
  * All timestamps conform strictly to RFC 3339 UTC.

---

### 2.2 `globalnews-storage` (Persistence & Search Core)
* **Role:** Manages the embedded SQLite database, schema migrations, and full-text search.
* **Dependencies:** `globalnews-contracts`, `rusqlite` (with `bundled` and `modern_sqlite` features), `thiserror`, `tracing`, `chrono`, `uuid`, `tokio`.
* **Architectural Boundaries:**
  1. **Bundled SQLite:** Eliminates system library version drift. Compiles the identical SQLite C engine (3.45+) into the binary on Windows, macOS, Linux, Android, and iOS.
  2. **WAL & FTS5 Activation:** Sets `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, `PRAGMA busy_timeout = 5000;`, and creates FTS5 virtual tables (`fts_events`, `fts_articles`) for sub-millisecond BM25 keyword search.
  3. **Migration Runner:** Embedded SQL files (`001_initial_schema.sql`) applied sequentially within atomic transactions, tracking state in `schema_migrations`.
  4. **Single-Writer Safety:** Wraps the write connection in a dedicated mutex (`Mutex<Connection>`), guaranteeing serialization of write transactions while allowing concurrent non-blocking readers.

---

### 2.3 `globalnews-platform` (Native Integration Boundaries)
* **Role:** Isolates operating-system-specific APIs and hardware lifecycle behaviors from the portable intelligence core.
* **Dependencies:** `globalnews-contracts`, `globalnews-storage`, `thiserror`, `tracing`.
* **Architectural Boundaries:**
  * Defines the `PlatformAdapter` trait:
    * `name(&self) -> &'static str`
    * `data_dir(&self) -> PathBuf`
    * `cache_dir(&self) -> PathBuf`
    * `is_metered_network(&self) -> bool`
    * `is_on_battery(&self) -> bool`
    * `show_notification(&self, title: &str, body: &str) -> Result<(), PlatformError>`
    * `schedule_background_sync(&self, interval_mins: u32) -> Result<(), PlatformError>`
  * Provides `DesktopAdapter` for desktop environments (standard filesystem directories, system tray hooks).
  * Provides `MobileAdapter` for mobile environments (scoped storage paths, JNI / Swift native bridge declarations).

---

### 2.4 `globalnews-core` (Portable Engine Orchestration)
* **Role:** Orchestrates state management, configuration, logging, and benchmarking.
* **Dependencies:** `globalnews-contracts`, `globalnews-storage`, `globalnews-platform`, `thiserror`, `tracing`, `tracing-subscriber`, `tokio`, `serde`, `serde_json`, `chrono`, `uuid`, `libc`.
* **Architectural Boundaries:**
  * `EngineState`: Thread-safe container (`Arc<DatabaseManager>`, `Arc<dyn PlatformAdapter>`, `CoreConfig`).
  * `BenchmarkRunner`: Infrastructure measuring database initialization time, startup latency, serialization throughput, and memory RSS on physical hardware.

---

### 2.5 `globalnews-app` (Tauri 2 Native Application Shell)
* **Role:** Serves as the user-facing application bundle and native IPC bridge.
* **Dependencies:** `globalnews-contracts`, `globalnews-storage`, `globalnews-core`, `globalnews-platform`, `serde`, `serde_json`, `thiserror`, `tracing`, `tokio`.
* **Architectural Boundaries:**
  * Zero Localhost Network Sockets: Commands communicate with the core via **in-process native message passing** (`commands::handle_get_health`, `commands::handle_search_events`).
  * Fine-grained capability definitions (`capabilities/default.json`) locking down permissions.

---

## 3. Dependency Graph & Layering Rules

```
        globalnews-app
          |          \
          |           \
          v            v
globalnews-core ---> globalnews-platform
          |
          v
globalnews-storage
          |
          v
globalnews-contracts
```

### Strict Layering Invariants:
1. `globalnews-contracts` must NEVER depend on any other workspace crate.
2. `globalnews-storage` may ONLY depend on `globalnews-contracts`.
3. `globalnews-platform` may depend on `globalnews-contracts` and `globalnews-storage`.
4. `globalnews-core` may depend on `contracts`, `storage`, and `platform`.
5. Circular dependencies are strictly forbidden by Cargo workspace resolution.
