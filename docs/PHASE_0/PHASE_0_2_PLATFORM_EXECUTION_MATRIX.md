# GLOBALNEWS — Phase 0.2 Platform Execution Matrix

**Document ID:** `GN-DOC-P02-003`  
**Phase:** Phase 0.2 (Cross-Platform Architecture Reconciliation)  
**Status:** Approved Technical Execution Specification  
**Date:** 2026-11-09  

---

## 1. Master Cross-Platform Execution Architecture

GLOBALNEWS establishes a unified operational model across all five platforms by deploying **Tauri 2.0** as the application shell and **`globalnews-core` (Rust)** as the embedded intelligence engine:

```
+---------------------------------------------------------------------------------------------------+
|                        CROSS-PLATFORM ARCHITECTURAL TAXONOMY                                      |
+---------------------------------------------------------------------------------------------------+
|  [ SHARED FRONTEND (HTML5 / CSS / Three.js / Three-Globe / WebGL) ]                               |
|  - 100% Shared UI code across Desktop, Android, and iOS                                           |
|  - Responsive: Mouse/Keyboard controls (Desktop) <---> Multi-touch gestures (Mobile)              |
|  - Level of Detail (LOD): High-DPI 60 FPS (Desktop GPU) <---> Battery-optimized LOD (Mobile GPU)  |
+---------------------------------------------------------------------------------------------------+
                                                  |
                                                  | Native Asynchronous Message Passing (IPC)
                                                  | Tauri 2 Commands & Event Streams
                                                  v
+---------------------------------------------------------------------------------------------------+
|  [ TAURI 2.0 NATIVE PLATFORM SHELL ]                                                              |
|  +----------------------------+  +----------------------------+  +-----------------------------+  |
|  | Windows / macOS / Linux    |  | Android (Kotlin / NDK)     |  | iOS (Swift / XCFramework)   |  |
|  | - System Tray & App Window |  | - WorkManager (Periodic)   |  | - BGTaskScheduler (Refresh) |  |
|  | - Direct In-Process Tokio  |  | - Optional Foreground Svc  |  | - Local UserNotifications   |  |
|  | - Native Desktop Alerts    |  | - Notification Channels    |  | - App Lifecycle Handler     |  |
|  +----------------------------+  +----------------------------+  +-----------------------------+  |
|                                                |                                                  |
|                                                v                                                  |
|  +---------------------------------------------------------------------------------------------+  |
|  | [ UNIFIED RUST CORE (globalnews-core) ]                                                     |  |
|  | - Ingestion & Parsing (USGS, GDACS, ReliefWeb, UN News, Wire RSS via reqwest / quick-xml)   |  |
|  | - Security Gating (SSRF DNS validator, XXE immunity, Size capping)                         |  |
|  | - Intelligence (MinHash/LSH dedup, Toponym resolution, Two-tier story clustering)           |  |
|  | - Local Storage (rusqlite WAL mode, FTS5 BM25 search, 3-tier retention engine)              |  |
|  +---------------------------------------------------------------------------------------------+  |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Desktop Execution Model (Windows, macOS, Linux)

```
+---------------------------------------------------------------------------------------------------+
|                               DESKTOP EXECUTION MATRIX                                            |
+--------------------------+------------------------------------------------------------------------+
| Dimension                | Architectural Implementation                                           |
+--------------------------+------------------------------------------------------------------------+
| **Lifecycle & Daemon**   | Runs in-process via multi-threaded Tokio runtime. When window is       |
|                          | closed, app minimizes to System Tray to maintain background monitoring.|
| **Startup & Shutdown**   | Optional OS auto-start on login (via `tauri-plugin-autostart`).        |
|                          | Clean SIGINT/SIGTERM handlers flush SQLite WAL checkpoints.           |
| **Background Ingestion** | Async Tokio worker loops every 5m (AC power) or 30m (Battery power).   |
| **Database Access**      | Local `globalnews.db` in standard user data directory. Strict WAL mode.|
| **IPC & Auth**           | Tauri 2 In-Process IPC (`invoke` / `emit`). Zero localhost network    |
|                          | sockets; completely immune to DNS rebinding and port conflicts!        |
| **Notifications**        | Native OS desktop banners via `tauri-plugin-notification`.             |
| **Application Updates**  | Cryptographically signed updates via `tauri-plugin-updater` (Ed25519). |
| **Crash Recovery**       | SQLite transactional ACID safety; automatic WAL recovery on reboot.    |
+--------------------------+------------------------------------------------------------------------+
```

---

## 3. Mobile Execution Model: Android

Android provides flexible scheduling capabilities through Google Play Services and the Android Jetpack ecosystem.

### 3.1 Background Scheduling via WorkManager
* **Periodic Work Scheduling:** Ingestion runs via `androidx.work.PeriodicWorkRequest` with an interval of **15 to 30 minutes**.
* **Work Constraints:**
  ```kotlin
  val constraints = Constraints.Builder()
      .setRequiredNetworkType(NetworkType.CONNECTED)
      .setRequiresBatteryNotLow(true)
      .build()
  ```
* **Execution Flow:** `WorkManager` triggers a native Kotlin worker that invokes `globalnews_core::sync_feeds()` via JNI. The Rust core executes network polling, deduplication, and SQLite commits in $<1.5\text{ seconds}$, then immediately terminates, yielding execution before the Android battery watchdog triggers.

### 3.2 Optional High-Priority Alert Monitoring (Foreground Service)
* For users requiring immediate breaking earthquake and disaster alerts (USGS/GDACS), an optional **Foreground Service** can be enabled in settings.
* Displays a low-profile ongoing notification: *"GlobalNews: Monitoring global hazard telemetry"*.
* Allows reliable real-time socket connections or 3-minute poll cycles without being killed by OEM battery savers (Samsung OneUI, Xiaomi MIUI).

### 3.3 Storage & Scoped Filesystem
* Database stored in private internal storage: `context.filesDir.resolve("globalnews.db")`.
* Storage quota enforced at $250\text{ MB}$ default on mobile; automated LRU vacuuming occurs when device storage is constrained.

---

## 4. Mobile Execution Model: iOS

Operating under iOS requires strict adherence to Apple's design philosophy and App Store Review Guidelines.

### 4.1 The iOS Reality: Honest Architectural Limitations
* **NEVER PROMISE CONTINUOUS STREAMING IN THE BACKGROUND ON iOS.**
* iOS does not allow non-audio, non-VoIP, non-navigation applications to maintain continuous background threads or loopback network daemons.
* GLOBALNEWS on iOS employs a **Two-Tier Synchronization Model**:
  1. **Opportunistic Background Refresh:** When granted by the iOS system.
  2. **Instant Foreground Catch-Up Sync:** Triggered immediately whenever the user opens or unlocks the application.

### 4.2 Background Scheduling via `BGTaskScheduler`
* **Task Registration:** Registers `com.globalnews.app.refresh` in `Info.plist` under `BGTaskSchedulerPermittedIdentifiers`.
* **Scheduling:** When the app enters the background, it requests a refresh task:
  ```swift
  let request = BGAppRefreshTaskRequest(identifier: "com.globalnews.app.refresh")
  request.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60) // 15 mins
  try? BGTaskScheduler.shared.submit(request)
  ```
* **Execution Budget Compliance:**
  * When iOS launches the task, the app has **at most 30 seconds**.
  * The Swift handler calls `globalnews_core_sync()` via C-FFI.
  * In compiled Rust, fetching feeds, parsing XML/GeoJSON, and writing to SQLite executes in **$< 1.8\text{ seconds}$** with a memory peak of **$< 20\text{ MB}$ RSS**.
  * The task explicitly calls `task.setTaskCompleted(success: true)` within 3 seconds, guaranteeing Apple's watchdog never terminates the app.

### 4.3 Foreground Catch-Up Synchronization
* Listens to `UIApplication.willEnterForegroundNotification`.
* Immediately queries Tier 1 disaster and wire endpoints, computes delta since last sync timestamp, updates SQLite, and streams new markers into the 3D globe via Tauri IPC in $< 1.0\text{ second}$.

---

## 5. Shared Application Interface & WebGL Adaptation

### 5.1 Single Shared UI Codebase
The entire user interface is built as a single responsive web application (HTML5, Vanilla CSS, Three.js, Three-Globe):
* **Desktop Experience:** Full mouse orbit controls, right-click context menus, multi-window support, expandable multi-column intelligence dossier drawer, hotkey shortcuts.
* **Mobile Experience:** Single-finger globe rotation, two-finger pinch-to-zoom, bottom sheet modal drawer for event intelligence, haptic feedback on marker tap.

### 5.2 GPU & Level of Detail (LOD) Adaptation
Mobile GPUs (e.g. Apple A-series / Qualcomm Adreno) have distinct thermal and battery throttles compared to desktop chips:
* **Desktop LOD:** Atmosphere Fresnel glow shader, 64-segment sphere geometry, dynamic pulsing rings, 60 FPS VSync locked.
* **Mobile LOD:** Adaptive framerate: 60 FPS during touch interaction, dropping to **15 FPS or static render when the globe is stationary** to save battery. Atmosphere shader simplified to low-overhead vertex falloff.

---

## 6. Synthesis: Platform Execution Matrix

```
+-----------------------------------------------------------------------------------------------------------------------------+
|                                           CROSS-PLATFORM MATRIX SUMMARY                                                     |
+---------------------+-------------------+-------------------+-------------------+-------------------+-------------------+
| Feature / Subsystem | Windows Desktop   | macOS Desktop     | Linux Desktop     | Android Mobile    | iOS Mobile        |
+---------------------+-------------------+-------------------+-------------------+-------------------+-------------------+
| **Binary Format**   | NSIS / MSI EXE    | Universal DMG     | AppImage / .deb   | APK / AAB (Store) | IPA (App Store)   |
| **Core Runtime**    | Compiled Rust lib | Compiled Rust lib | Compiled Rust lib | Rust JNI (.so)    | Rust XCFramework  |
| **Webview Engine**  | Edge WebView2     | Apple WKWebView   | WebKitGTK         | Android WebView   | Apple WKWebView   |
| **IPC Model**       | Native Tauri IPC  | Native Tauri IPC  | Native Tauri IPC  | Native Tauri IPC  | Native Tauri IPC  |
| **Background Sync** | Tokio Tray Worker | Tokio / launchd   | Tokio / systemd   | WorkManager (15m) | BGAppRefreshTask  |
| **Continuous Mode** | Supported (Tray)  | Supported (Tray)  | Supported (Daemon)| Opt-in Foregnd Svc| NOT SUPPORTED     |
| **Database**        | SQLite FTS5 (WAL) | SQLite FTS5 (WAL) | SQLite FTS5 (WAL) | SQLite FTS5 (WAL) | SQLite FTS5 (WAL) |
| **Storage Quota**   | 500 MB (Default)  | 500 MB (Default)  | 500 MB (Default)  | 250 MB (Default)  | 250 MB (Default)  |
| **Memory Ceiling**  | <120 MB RAM       | <120 MB RAM       | <120 MB RAM       | <60 MB RAM        | <45 MB RAM        |
+---------------------+-------------------+-------------------+-------------------+-------------------+-------------------+
```

