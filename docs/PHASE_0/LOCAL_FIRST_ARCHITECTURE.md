# GLOBALNEWS — Local-First Application Architecture

**Document ID:** `GN-DOC-P0-007`  
**Phase:** Phase 0 (Research & Feasibility)  
**Status:** Approved Architectural Baseline  
**Inspection Date:** 2026-11-09  
**Core Decision:** Decoupled Hybrid Architecture (Lightweight Local Daemon + High-Performance Webview/PWA Client).

---

## 1. Architectural Paradigms Evaluation

To determine the optimal foundation for GLOBALNEWS, three deployment models were subjected to rigorous comparative engineering analysis:

```
+---------------------------------------------------------------------------------------------------+
| MODEL A: PURE PROGRESSIVE WEB APP (In-Browser Execution Only)                                     |
| [ Browser Tab ] -> [ Service Worker / Web Workers ] -> [ OPFS + SQLite WASM ]                   |
+---------------------------------------------------------------------------------------------------+
                                                  vs
+---------------------------------------------------------------------------------------------------+
| MODEL B: MONOLITHIC TAURI / ELECTRON DESKTOP (All-in-one Native Binary)                           |
| [ Native Process ] -> [ Embedded Rust / Node Core ] -> [ Native SQLite ] -> [ Webview Window ]    |
+---------------------------------------------------------------------------------------------------+
                                                  vs
+---------------------------------------------------------------------------------------------------+
| MODEL C: DECOUPLED HYBRID ARCHITECTURE (Recommended Master Architecture)                          |
| [ Headless Local Engine Daemon (localhost:7474) ] <--- SSE/IPC ---> [ PWA or Tauri Webview UI ]    |
+---------------------------------------------------------------------------------------------------+
```

### 1.1 Comparative Engineering Matrix

| Architectural Criterion | Model A: Pure Browser PWA | Model B: Monolithic Desktop | Model C: Decoupled Hybrid (Recommended) |
| :--- | :--- | :--- | :--- |
| **Bypassing Browser CORS** | **FATAL FAILURE** (Cannot fetch RSS directly) | Excellent (Native HTTP clients) | **Excellent** (Daemon fetches feeds directly) |
| **24/7 Background Ingestion** | **FATAL FAILURE** (OS suspends SW in <60s) | Good (Runs while app window open) | **Superior** (OS background daemon/launchd) |
| **Storage Durability** | At risk (Browser may evict OPFS) | High (Native file system) | **High** (Deterministic SQLite file) |
| **NLP Execution Efficiency** | Slow (WASM single-thread overhead) | Fast (Native C/Rust bindings) | **Fast** (Native Python / Node / C extensions) |
| **Memory Consumption** | Medium (Browser overhead) | Low (Tauri: ~50MB) / High (Electron) | **Minimal** (Daemon ~80MB + Webview ~60MB) |
| **Zero-Install Web Access** | Instant | Requires installation | **Instant** (Localhost UI accessible via any browser) |
| **Hardware Portability** | Universal | Requires per-OS compile (Rust/C++) | **Universal** (Runs on Python/Node runtimes) |
| **Battery Impact Control** | None (Browser controls scheduling) | Moderate | **Complete** (OS-level `nice`, I/O throttling) |

---

## 2. Why a Pure Browser PWA is Fundamentally Disqualified

While a browser-only PWA is superficially appealing due to zero-installation friction, deep technical research reveals **three insurmountable operational blockers**:

### 2.1 The Cross-Origin Resource Sharing (CORS) Wall
Web browsers strictly forbid client-side JavaScript (`fetch()` or `XMLHttpRequest`) from reading responses from arbitrary third-party domains (e.g. `bbc.co.uk`, `reuters.com`, `gdacs.org`) unless the server explicitly sets the header:
```http
Access-Control-Allow-Origin: *
```
Over **90% of global news RSS feeds and institutional publishers omit CORS headers** or restrict them. A pure PWA would therefore be forced to route all news traffic through an external third-party CORS proxy server (e.g. `cors-anywhere`), instantly destroying the project's core principles:
* Creates a centralized single point of failure.
* Exposes user reading habits and polling requests to a centralized server.
* Incurs proxy bandwidth costs and rate-limiting blocks.

### 2.2 Aggressive Mobile & Desktop Lifecycle Throttling
Modern operating systems (macOS Sonoma/Sequoia, Windows 11, iOS, Android) actively preserve battery by aggressively terminating background tabs and Service Workers:
* When a browser tab is minimized or backgrounded, `setInterval` and `setTimeout` timers are clamped to at most 1 execution per minute or suspended entirely.
* Service Workers are actively terminated by Chrome and Safari after **30 to 60 seconds of inactivity**.
* Reliable, continuous monitoring of breaking global events (such as real-time earthquake alerts) is mathematically impossible in a pure background browser thread.

### 2.3 Unpredictable Storage Eviction
Under browser storage specifications, Origin Private File System (OPFS) and IndexedDB are subject to **automated eviction** when the host operating system signals low disk space (as seen in our host environment with 92% capacity used). The browser may silently wipe the local database without explicit user consent.

---

## 3. The Recommended Master Architecture: Decoupled Hybrid

GLOBALNEWS adopts a clean architectural separation between the **Background Ingestion/Intelligence Engine (The Daemon)** and the **Interactive 3D Cartographic Shell (The Client)**:

```
+---------------------------------------------------------------------------------------------------+
|                        GLOBALNEWS DECOUPLED HYBRID ARCHITECTURE                                   |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|  [ THE CLIENT INTERFACE ] (Runs in Browser PWA or Tauri WKWebView)                                |
|  +---------------------------------------------------------------------------------------------+  |
|  | - Three-Globe 3D Rendering (WebGL / InstancedMesh)                                          |  |
|  | - Hotspot Interaction, Camera OrbitControls & Zoom LOD                                      |  |
|  | - Event Intelligence Dossier Drawer & Search UI                                             |  |
|  | - Server-Sent Events (SSE) Client: Listens to /events/live                                  |  |
|  +---------------------------------------------------------------------------------------------+  |
|                                                ^                                                  |
|                        HTTP / SSE / REST       |  Zero-Latency Local IPC                          |
|                       (http://127.0.0.1:7474)  v                                                  |
|  +---------------------------------------------------------------------------------------------+  |
|  [ THE INTELLIGENCE DAEMON ] (Headless Native Background Process)                                 |
|  | - Ingestion Scheduler (Polite RSS, GDACS, USGS, GDELT fetchers)                             |  |
|  | - Security & Sanitization Layer (SSRF validation, defusedxml)                               |  |
|  | - Lightweight NLP Pipeline (FastText, Trafilatura, spaCy NER, TextRank)                     |  |
|  | - MinHash / LSH Deduplication & Story Clustering Engine                                     |  |
|  | - Local SQLite 3 FTS5 Database (Storage, full-text BM25 index, schema migrations)           |  |
|  | - Resource & Battery Manager (macOS sleep watcher, quota enforcement)                       |  |
|  +---------------------------------------------------------------------------------------------+  |
|                                                                                                   |
+---------------------------------------------------------------------------------------------------+
```

### 3.1 Components of the Decoupled System

#### A. The GlobalNews Daemon (`gn-core`)
* **Runtime:** Lightweight process executed via Node.js or Python (both verified available on host: Node v22.22, Python 3.13).
* **Network Binding:** Binds strictly to `127.0.0.1:7474` (Loopback only; inaccessible from external networks).
* **Communication Protocol:**
  * `GET /api/v1/events?bbox=...&min_sig=...` (Spatial & significance filtering).
  * `GET /api/v1/events/:id` (Detailed intelligence dossier).
  * `GET /api/v1/search?q=...` (Sub-millisecond SQLite FTS5 search).
  * `GET /api/v1/stream` (Persistent Server-Sent Events channel streaming new validated events directly into the 3D globe in real time).
* **Daemon Management:** Managed natively on macOS via `launchd` plist (or running as a standard CLI process `gn-daemon start`).

#### B. The Frontend Shell (`gn-ui`)
* **Technology:** HTML5, CSS3, Three.js, Three-Globe.
* **Packaging Flexibility:**
  1. **Mode 1 (Instant Webview / PWA):** User opens `http://localhost:7474` in Safari, Chrome, or Firefox, with full offline PWA caching of static assets.
  2. **Mode 2 (Tauri Desktop App):** Packaged as a native macOS application bundle using WKWebView, embedding or spawning the daemon automatically upon launch.

---

## 4. Offline Capability & Data Portability

Because all intelligence is persisted in a single local SQLite database file (`~/.globalnews/globalnews.db`):
* **Full Offline Operability:** When disconnected from the internet (e.g. during flights, remote travel, or network outages), the entire 3D Earth, full-text search, historical timeline playback, and event dossiers remain 100% functional.
* **Zero Cloud Lock-in:** The entire user database can be backed up, inspected with standard tools (`sqlite3`), exported to GeoJSON, or transferred between machines with zero friction.

---

## 5. Optional Privacy-Preserving Shared Intelligence

To reduce duplicated processing across community users without compromising privacy, GLOBALNEWS designs an optional peer-to-peer or hash-sync distribution mechanism:

```
+-----------------------------------+               +-----------------------------------+
|          NODE ALICE               |               |            NODE BOB               |
| Ingests RSS -> Resolves Event     |               | Receives signed metadata packet:  |
| Hash: sha256(Title + Date + GPE)  | -- Gossip --> | - Event Hash                      |
| Impact: 7.2 | Geo: [35.68, 139.76]|               | - Lat/Lng & Impact Score          |
+-----------------------------------+               +-----------------------------------+
```

### Privacy & Security Guarantees
1. **Zero Tracking:** Nodes only exchange cryptographic hashes and normalized event metadata (coordinates, impact score, category code).
2. **Zero Reading History:** Individual article reads, clicks, search queries, and bookmarked events are never transmitted over the wire.
3. **Sybil Resistance:** Metadata packets require cryptographic signature verification against verified source URLs, preventing malicious nodes from injecting fictitious events.

