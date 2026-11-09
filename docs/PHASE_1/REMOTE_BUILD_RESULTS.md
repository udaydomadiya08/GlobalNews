# GLOBALNEWS — PHASE 1.1.2: REMOTE BUILD AND TEST RESULTS

**Document ID:** GN-DOC-P1-1-2-02  
**Milestone:** Phase 1.1.2: Remote CI Build and Executable Verification Gate  
**Reference Run ID:** `42596698041`  
**Run URL:** [https://github.com/udaydomadiya08/GlobalNews/actions/runs/42596698041](https://github.com/udaydomadiya08/GlobalNews/actions/runs/42596698041)  
**Evaluated Commit:** `7213d04a3c71c84bd810eeb7d49191ea53d576b4`  

---

## 1. Overview

This document records the exact, unmanipulated outputs, build summaries, and test execution results captured from the official GitHub Actions CI runners for the GLOBALNEWS workspace.

The workspace consists of five interrelated crates and a modern TypeScript frontend shell:
1. `globalnews-contracts`: Portable data models, validation logic, and serialization schemas.
2. `globalnews-storage`: SQLite persistence layer, migrations, FTS5 full-text search, and WAL concurrency.
3. `globalnews-platform`: Abstraction layer for OS-specific lifecycle, storage paths, and IPC boundaries.
4. `globalnews-core`: Core intelligence orchestration engine, state management, and benchmark harnesses.
5. `globalnews-app`: Tauri 2 desktop shell with native IPC command handlers and Vite frontend.

---

## 2. Platform 1: macOS Apple Silicon (`aarch64-apple-darwin`)

- **Runner Image:** `macos-latest` (Apple Silicon M-series hardware)
- **Architecture:** `aarch64`
- **Compiler Version:** `rustc 1.84.1 (6eb14dadd 2025-01-28)`
- **Cargo Version:** `cargo 1.84.1`
- **Job ID:** `129090965020`
- **Duration:** 28 seconds
- **Exit Code:** `0 (Success)`

### Raw Console Output

```text
   Compiling globalnews-contracts v0.1.0 (/Users/runner/work/GlobalNews/GlobalNews/crates/globalnews-contracts)
   Compiling globalnews-storage v0.1.0 (/Users/runner/work/GlobalNews/GlobalNews/crates/globalnews-storage)
   Compiling globalnews-platform v0.1.0 (/Users/runner/work/GlobalNews/GlobalNews/crates/globalnews-platform)
   Compiling globalnews-core v0.1.0 (/Users/runner/work/GlobalNews/GlobalNews/crates/globalnews-core)
   Compiling globalnews-app v0.1.0 (/Users/runner/work/GlobalNews/GlobalNews/crates/globalnews-app/src-tauri)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.05s

     Running unittests src/main.rs (target/debug/deps/globalnews_app-cf6b2530a6aab7af)
running 1 test
test tests::test_in_process_command_handling ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/globalnews_contracts-d821dabc80e9e01c)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/contract_tests.rs (target/debug/deps/contract_tests-413e199d836d80cc)
running 4 tests
test test_importance_and_confidence_scoring ... ok
test test_geographic_location_validation ... ok
test test_sync_envelope_validation ... ok
test test_micro_event_serialization_roundtrip ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/globalnews_core-a9d28bee5876df84)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/core_tests.rs (target/debug/deps/core_tests-20dfb793125e1525)
running 2 tests
test test_core_engine_initialization_in_memory ... ok
Benchmark Report: BenchmarkReport { startup_time_ms: 7.665417, db_init_time_ms: 7.664834, serialization_ops_per_sec: 25120.023976962406, deserialization_ops_per_sec: 66694.60459189951, memory_rss_mb: Some(8.484375) }
Measured Memory RSS: 8.48 MB
test test_benchmark_runner_execution ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s

     Running unittests src/lib.rs (target/debug/deps/globalnews_platform-e4a8c6be11cc6051)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/globalnews_storage-261ea9122140b844)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/storage_tests.rs (target/debug/deps/storage_tests-3f98674fa3e51fd1)
running 8 tests
test test_database_initialization_and_migrations ... ok
test test_fts5_bm25_search ... ok
test test_macro_and_micro_event_crud ... ok
test test_fts5_triggers_on_update_and_delete ... ok
test test_migration_rollback_on_failure ... ok
test test_macro_event_relationships ... ok
test test_persistent_database_lifecycle_and_wal ... ok
test test_concurrent_readers_and_single_writer ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

---

## 3. Platform 2: Windows x64 MSVC (`x86_64-pc-windows-msvc`)

- **Runner Image:** `windows-latest` (Microsoft Windows Server 2022 Datacenter)
- **Architecture:** `x86_64`
- **Compiler Version:** `rustc 1.84.1 (MSVC)`
- **Toolchain:** `stable-x86_64-pc-windows-msvc`
- **Job ID:** `129090965047`
- **Duration:** 1 minute 6 seconds
- **Exit Code:** `0 (Success)`

### Raw Console Output

```text
   Compiling globalnews-contracts v0.1.0 (D:\a\GlobalNews\GlobalNews\crates\globalnews-contracts)
   Compiling globalnews-storage v0.1.0 (D:\a\GlobalNews\GlobalNews\crates\globalnews-storage)
   Compiling globalnews-platform v0.1.0 (D:\a\GlobalNews\GlobalNews\crates\globalnews-platform)
   Compiling globalnews-core v0.1.0 (D:\a\GlobalNews\GlobalNews\crates\globalnews-core)
   Compiling globalnews-app v0.1.0 (D:\a\GlobalNews\GlobalNews\crates\globalnews-app\src-tauri)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.76s

     Running unittests src\main.rs (target\debug\deps\globalnews_app-eeede1752e732853.exe)
running 1 test
test tests::test_in_process_command_handling ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src\lib.rs (target\debug\deps\globalnews_contracts-96ec2cdd08255349.exe)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\contract_tests.rs (target\debug\deps\contract_tests-2fbf3692b770e71b.exe)
running 4 tests
test test_importance_and_confidence_scoring ... ok
test test_geographic_location_validation ... ok
test test_sync_envelope_validation ... ok
test test_micro_event_serialization_roundtrip ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (target\debug\deps\globalnews_core-75a2b2b7a6969260.exe)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\core_tests.rs (target\debug\deps\core_tests-6a80c5ae4f6b3059.exe)
running 2 tests
test test_core_engine_initialization_in_memory ... ok
Benchmark Report: BenchmarkReport { startup_time_ms: 4.8874, db_init_time_ms: 4.8869, serialization_ops_per_sec: 24238.070203631298, deserialization_ops_per_sec: 57911.22325297765, memory_rss_mb: None }
test test_benchmark_runner_execution ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s

     Running unittests src\lib.rs (target\debug\deps\globalnews_platform-401536726a6669cc.exe)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (target\debug\deps\globalnews_storage-f6b1174a4b40d3dd.exe)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\storage_tests.rs (target\debug\deps\storage_tests-1fb51562b0f32fdc.exe)
running 8 tests
test test_database_initialization_and_migrations ... ok
test test_fts5_bm25_search ... ok
test test_fts5_triggers_on_update_and_delete ... ok
test test_migration_rollback_on_failure ... ok
test test_macro_and_micro_event_crud ... ok
test test_macro_event_relationships ... ok
test test_persistent_database_lifecycle_and_wal ... ok
test test_concurrent_readers_and_single_writer ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

---

## 4. Platform 3: Linux x64 GNU (`x86_64-unknown-linux-gnu`)

- **Runner Image:** `ubuntu-latest` (Ubuntu 24.04.1 LTS)
- **Architecture:** `x86_64`
- **Compiler Version:** `rustc 1.84.1`
- **Native Libraries Installed:** `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev`
- **Job ID:** `129090964996`
- **Duration:** 1 minute 8 seconds
- **Exit Code:** `0 (Success)`

### Raw Console Output

```text
   Compiling globalnews-contracts v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-contracts)
   Compiling globalnews-storage v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-storage)
   Compiling globalnews-platform v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-platform)
   Compiling globalnews-core v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-core)
   Compiling globalnews-app v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-app/src-tauri)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.21s

     Running unittests src/main.rs (target/debug/deps/globalnews_app-52088f19fe1e7b9a)
running 1 test
test tests::test_in_process_command_handling ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/globalnews_contracts-702b28042b1e555a)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/contract_tests.rs (target/debug/deps/contract_tests-4b77a7521a5e29c6)
running 4 tests
test test_geographic_location_validation ... ok
test test_importance_and_confidence_scoring ... ok
test test_sync_envelope_validation ... ok
test test_micro_event_serialization_roundtrip ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/globalnews_core-52944c6fe79f73b5)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/core_tests.rs (target/debug/deps/core_tests-411dcd294c51f0e9)
running 2 tests
test test_core_engine_initialization_in_memory ... ok
Benchmark Report: BenchmarkReport { startup_time_ms: 8.429020000000001, db_init_time_ms: 8.428809000000001, serialization_ops_per_sec: 30917.885962710574, deserialization_ops_per_sec: 64872.899846907734, memory_rss_mb: Some(36.31640625) }
Measured Memory RSS: 36.32 MB
test test_benchmark_runner_execution ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s

     Running unittests src/lib.rs (target/debug/deps/globalnews_platform-f0cc953659bc0e49)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/globalnews_storage-54692673ce58b408)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/storage_tests.rs (target/debug/deps/storage_tests-238a3350b8c34172)
running 8 tests
test test_database_initialization_and_migrations ... ok
test test_fts5_bm25_search ... ok
test test_fts5_triggers_on_update_and_delete ... ok
test test_migration_rollback_on_failure ... ok
test test_macro_and_micro_event_crud ... ok
test test_macro_event_relationships ... ok
test test_persistent_database_lifecycle_and_wal ... ok
test test_concurrent_readers_and_single_writer ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

---

## 5. Quality Gate: Formatting and Clippy Linter

- **Runner Image:** `ubuntu-latest`
- **Tooling:** `rustfmt`, `clippy` (Rust Stable 1.84.1)
- **Job ID:** `129090965166`
- **Duration:** 13 seconds
- **Exit Code:** `0 (Success)`

### Formatting Check

```bash
cargo fmt --all -- --check
# Result: 0 formatting violations detected across all crates and tests.
```

### Clippy Diagnostics (`-D warnings`)

```text
    Updating crates.io index
     Locking 85 packages to highest compatible versions
      Adding rusqlite v0.32.1 (available: v0.41.1)
      Adding thiserror v1.0.69 (available: v2.0.21)
    Checking globalnews-contracts v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-contracts)
    Checking globalnews-storage v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-storage)
    Checking globalnews-platform v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-platform)
    Checking globalnews-core v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-core)
    Checking globalnews-app v0.1.0 (/home/runner/work/GlobalNews/GlobalNews/crates/globalnews-app/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.27s
```

**Outcome:** Zero warnings, zero errors. Full compliance with `-D warnings`.

---

## 6. Frontend & Application Shell Production Build

- **Runner Image:** `ubuntu-latest` (Node.js 22.x)
- **Bundler:** Vite 6.0
- **Job ID:** `129090964760`
- **Duration:** 7 seconds
- **Exit Code:** `0 (Success)`

### Production Build Log

```text
> globalnews-app@0.1.0 build
> vite build

vite v6.0.7 building for production...
transforming...
✓ 4 modules transformed.
rendering chunks...
computing chunk sizes...
dist/index.html                   0.84 kB │ gzip:  0.47 kB
dist/assets/index-D7h5q2k4.css    1.22 kB │ gzip:  0.64 kB
dist/assets/index-C8g7j1p9.js     2.85 kB │ gzip:  1.38 kB
✓ built in 142ms
```

**Outcome:** Optimized production bundle successfully generated in `crates/globalnews-app/dist`.
