# GLOBALNEWS — PHASE 1.1.1: TAURI INTEGRATION VERIFICATION

**Date:** 2026-11-09  
**Status:** Architecture Audited & Scaffolding Verified  
**Crate:** `crates/globalnews-app/src-tauri/`  
**Framework Target:** Tauri 2.x

---

## 1. Factual Implementation Scope

In accordance with Phase 1.1 scope boundaries, `globalnews-app` establishes the **in-process native IPC command layer and application shell scaffolding**.

### Explicit Disclosure of Current State
- The Tauri CLI is not installed on this host (`tauri not found`).
- The crate `globalnews-app` currently depends on workspace crates (`contracts`, `storage`, `core`, `platform`) and serialization/logging utilities.
- It does **not** yet link `tauri = "2.x"` or execute `tauri::Builder::default().run()` in `main.rs`, because the desktop GUI webview container and frontend Vite/React/Three.js assets belong to Phase 3.
- The command layer in `src/commands.rs` implements typed, thread-safe command handlers that mirror Tauri's command pattern, ready to be wrapped with `#[tauri::command]` when Tauri 2 dependencies are pulled in.

---

## 2. In-Process IPC Command Audit

The IPC layer in `src/commands.rs` defines the communication protocol between the frontend and the Rust intelligence core:

```rust
// Core IPC Handlers Implemented
pub fn handle_get_health(state: &EngineState) -> HealthResponse;
pub fn handle_search_events(state: &EngineState, query: String, limit: Option<u32>) -> Result<Vec<SearchResult>, CommandError>;
pub fn handle_get_micro_event(state: &EngineState, id: String) -> Result<Option<MicroEvent>, CommandError>;
pub fn handle_get_macro_event(state: &EngineState, id: String) -> Result<Option<MacroEvent>, CommandError>;
```

### 2.1 Request Validation & Error Propagation
- **Empty Query Guard:** `handle_search_events` validates that `query.trim()` is not empty before initiating database search, returning typed `CommandError::InvalidRequest`.
- **Default Limit Clamping:** Search limit defaults to 20 if omitted (`limit.unwrap_or(20)`).
- **Error Serialization:** `CommandError` derives `Serialize, Deserialize, thiserror::Error`, allowing rich error strings to cross the IPC boundary to TypeScript.

### 2.2 Concurrency Correction (Phase 1.1.1)
- In Phase 1.1, read commands erroneously acquired the exclusive database write mutex (`state.db.with_writer`).
- In Phase 1.1.1, all three read commands were migrated to `state.db.with_reader`, ensuring that user search operations on the UI thread never block or contend with background data ingestion.

---

## 3. Capability Configuration

Tauri 2 enforces a strict capability-based security model. The capability definition in `capabilities/default.json` establishes the permissions baseline:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities for GlobalNews desktop shell",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:event:default"
  ]
}
```

- Restricts native capabilities exclusively to the `"main"` window.
- Grants only `core:default` and `core:event:default` permissions, preventing unauthorized access to raw operating system APIs from webview scripts.

---

## 4. Lifecycle & Shutdown Behavior

1. **Engine Initialization:**
   `main.rs` initializes `EngineState::new_desktop()`, which resolves platform-specific paths (`DesktopAdapter::data_dir()`), configures SQLite WAL pragmas, and runs forward migrations.
2. **Graceful Teardown:**
   `DatabaseManager` and connections implement `Drop`, ensuring that when the application receives a termination signal (`SIGINT`, `SIGTERM`), all pending WAL frames are flushed and file locks are released cleanly without corruption.
