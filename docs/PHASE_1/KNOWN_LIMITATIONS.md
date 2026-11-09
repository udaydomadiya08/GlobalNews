# GLOBALNEWS — PHASE 1.1: KNOWN LIMITATIONS AND ARCHITECTURAL BOUNDARIES

**Date:** 2026-11-09  
**Status:** Approved for Foundation Milestone  
**Scope:** Honest technical limitations, platform SDK requirements, cross-compilation boundaries, and resource invariants across the 5 mandatory platform targets.

---

## 1. Cross-Platform Compilation & Toolchain Realities

While the GLOBALNEWS architecture is designed for 5 mandatory platform targets (macOS DMG, Windows EXE, Linux AppImage, Android APK/AAB, and iOS IPA), a single developer workstation or single-platform CI runner cannot natively build all five targets without host-specific toolchains, proprietary SDKs, and code signing identities.

### Target Classification Matrix

| Target Triple | Platform | Host Status | Toolchain Prerequisites | Build / Test Status in Phase 1.1 |
|---|---|---|---|---|
| `aarch64-apple-darwin` | macOS (Apple Silicon) | Native Host | macOS 14+, Xcode 15+, Apple Clang | **Compiled, Executed & Verified Locally** |
| `x86_64-pc-windows-msvc` | Windows 10/11 x64 | Cross / Remote CI | Windows SDK 10/11, MSVC v143+ or `cargo-xwin` | Configured in CI matrix; requires Windows host or `xwin` sysroot |
| `x86_64-unknown-linux-gnu` | Linux x86_64 | Cross / Remote CI | GCC/Clang, `libwebkit2gtk-4.1-dev`, `libssl-dev` | Configured in CI matrix; requires Ubuntu runner or Docker cross |
| `aarch64-linux-android` | Android (ARM64) | Cross-compiled | Android NDK r25b+, API Level 26+, `cargo-ndk` | Configured in Cargo/Tauri; requires NDK sysroot |
| `aarch64-apple-ios` | iOS (ARM64) | Native macOS Host | Xcode 15+, iOS 16+ SDK, Apple Developer Team ID | Configured via `tauri ios`; requires physical device / simulator |

### Hard Toolchain Boundaries

1. **iOS Code Signing & Provisioning:**
   - Compiling a signed `.ipa` cannot be done headlessly on Linux or Windows; it requires macOS with an active Apple Developer certificate and provisioning profile.
   - Rust static libraries (`libglobalnews_core.a`) can compile to `aarch64-apple-ios`, but assembling the Tauri 2 iOS app container requires Xcode CLI tools.

2. **Android NDK C-Binds (`rusqlite`):**
   - SQLite bundling via `rusqlite` with the `bundled` feature compiles SQLite from C source. Cross-compiling to `aarch64-linux-android` requires the Android NDK `clang` wrapper with `--target=aarch64-linux-android26`.

3. **Linux WebKitGTK Dependencies:**
   - Tauri 2 on Linux depends on `webkit2gtk-4.1` and `gtk3`. Cross-compiling from macOS to Linux GNU requires complex multiarch Debian/Ubuntu sysroots; therefore, Linux AppImage packaging is delegated to native Ubuntu 22.04 / 24.04 GitHub Actions runners.

4. **Windows MSVC Linking:**
   - Compiling for `x86_64-pc-windows-msvc` from macOS requires `xwin` to extract CRT and Windows SDK headers. For production release, native Windows runners (`windows-latest`) must be utilized.

---

## 2. In-Process Persistence & Concurrency Limitations

The storage engine (`globalnews-storage`) relies on embedded SQLite compiled with WAL (Write-Ahead Logging) mode and connection pooling via `r2d2`.

### Known Invariants & Edge Cases

1. **Single-Writer Constraint:**
   - SQLite in WAL mode permits unlimited concurrent readers but strictly **one writer** at any given millisecond.
   - While `r2d2` manages pool access and `PRAGMA busy_timeout = 5000` provides 5 seconds of backoff, massive parallel batch writes from multiple threads will serialize.
   - Bulk ingestion must use transactional batches (`BEGIN IMMEDIATE` ... `COMMIT`) rather than individual write operations.

2. **FTS5 Virtual Table Rebuilding:**
   - Full-text search indices (`fts_events`, `fts_articles`) use `content=` external content mapping with triggers.
   - If records are bulk-inserted bypassing triggers (e.g., raw SQLite commands), full-text indices will desynchronize until an explicit `INSERT INTO fts_events(fts_events) VALUES('rebuild')` is executed.

3. **Mobile Storage Footprint:**
   - On iOS and Android, sandbox disk space can be reclaimed by the OS if classified under temporary caches.
   - All critical databases are placed strictly within `AppDirs::data_dir` (`NSDocumentDirectory` / Android `filesDir`), never in cache directories.

---

## 3. Platform Execution & Lifecycle Limitations

The abstraction layer (`globalnews-platform`) defines separate desktop and mobile adapters to respect OS constraints:

1. **iOS Background Execution Impossibility:**
   - Apple iOS strictly terminates long-running background tasks after ~30 seconds (via Jetsam).
   - Any expectation of continuous background news crawling on iOS is physically impossible and violates App Store Review Guideline 2.5.2.
   - On iOS, news synchronization must run opportunistically via `BGAppRefreshTask` or upon foreground app launch.

2. **Android Doze & Battery Optimization:**
   - While Android supports `WorkManager` with `PeriodicWorkRequest`, execution intervals are flexible (minimum 15 minutes) and subject to system Doze states.
   - Heavy NLP clustering or entity extraction cannot be scheduled on battery-saver mode.

3. **Desktop Sleep / Wake Recovery:**
   - When a laptop closes its lid, ongoing Tokio async tasks are paused.
   - Upon wake, network sockets may be dead. The ingestion scheduler must implement stale connection detection and reconnection backoff.

---

## 4. Scope Exclusions in Phase 1.1

To prevent scope creep and ensure rock-solid architectural verification, the following modules are explicitly excluded from Phase 1.1:

- **No Real News Scraping or Feed Crawling:** No network scrapers or live RSS parsers are loaded. Feeds are tested using deterministic JSON fixture envelopes.
- **No 3D Globe WebGL Rendering:** Frontend Three.js / Three-Globe canvas rendering is scheduled for Phase 3. Phase 1 focuses exclusively on the Rust core, database, contracts, platform abstraction, and IPC command layer.
- **No Heavy NLP / Embedding Models:** Tokenizers, FastText, and embedding weights are scheduled for Phase 1.2 / Phase 2. Phase 1 implements lexical FTS5 BM25 search.
