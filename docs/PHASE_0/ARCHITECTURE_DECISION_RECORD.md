# GLOBALNEWS — Architecture Decision Records (ADRs)

**Document ID:** `GN-DOC-P0-010`  
**Phase:** Phase 0 (Research & Feasibility)  
**Status:** Approved Architectural Baseline  
**Inspection Date:** 2026-11-09  

This document logs the critical architectural choices governing the GLOBALNEWS platform, detailing the context, evaluation, decisions, and trade-offs.

---

## Index of Architecture Decision Records

* [ADR-001: Decoupled Local Daemon + Webview Architecture](#adr-001-decoupled-local-daemon--webview-architecture)
* [ADR-002: Three-Globe with Custom InstancedMesh Shaders for 3D Earth](#adr-002-three-globe-with-custom-instancedmesh-shaders-for-3d-earth)
* [ADR-003: Deterministic & Statistical NLP over Generative LLMs](#adr-003-deterministic--statistical-nlp-over-generative-llms)
* [ADR-004: MinHash/LSH + Incremental Online Centroid Story Clustering](#adr-004-minhashlsh--incremental-online-centroid-story-clustering)
* [ADR-005: SQLite 3 with FTS5 and BM25 Ranking as Primary Store](#adr-005-sqlite-3-with-fts5-and-bm25-ranking-as-primary-store)
* [ADR-006: Tiered 3-Stage Storage Retention Hierarchy](#adr-006-tiered-3-stage-storage-retention-hierarchy)
* [ADR-007: Multi-Source 5-Tier Ingestion Strategy](#adr-007-multi-source-5-tier-ingestion-strategy)
* [ADR-008: Significance-Driven Scoring Decoupled from Media Volume](#adr-008-significance-driven-scoring-decoupled-from-media-volume)
* [ADR-009: Offline Two-Stage Toponym Resolution via Local GeoNames](#adr-009-offline-two-stage-toponym-resolution-via-local-geonames)
* [ADR-010: Trafilatura and DefusedXML for Secure Ingestion](#adr-010-trafilatura-and-defusedxml-for-secure-ingestion)

---

### ADR-001: Decoupled Local Daemon + Webview Architecture

* **Status:** Accepted
* **Context:** The application requires 24/7 continuous news ingestion, background feed polling, local NLP extraction, and instant 3D visualization. A pure browser PWA fails because browsers enforce CORS (blocking 90% of direct RSS feeds) and terminate background Service Workers within 60 seconds. Electron is rejected due to excessive memory consumption (>300 MB base).
* **Decision:** Implement a **Decoupled Hybrid Architecture**:
  1. A headless local daemon (`gn-core`) running natively in the background on `127.0.0.1:7474`.
  2. A lightweight client shell (`gn-ui`) rendered in WKWebView (via Tauri or standard browser PWA) communicating via Server-Sent Events (SSE) and local REST endpoints.
* **Consequences:**
  * *Positive:* Bypasses browser CORS completely; enables true continuous background ingestion; operates within $<350\text{ MB}$ total RAM; supports zero-install browser access on localhost.
  * *Negative:* Requires running a local background process on the user's operating system.
  * *Neutral:* Requires managing local daemon lifecycle (`launchd` on macOS, systemd on Linux).

---

### ADR-002: Three-Globe with Custom InstancedMesh Shaders for 3D Earth

* **Status:** Accepted
* **Context:** The central interface is a realistic 3D Earth displaying up to 10,000 simultaneous glowing event hotspots at 60 FPS on integrated GPUs (Apple M1 7/8-core). Individual mesh objects create thousands of draw calls, stalling the GPU. CesiumJS is heavy (>3.8 MB WASM) and geared toward terrain GIS rather than abstract data intelligence.
* **Decision:** Adopt **Three-Globe** as the spherical cartographic foundation, and render all event hotspots using a single **`THREE.InstancedMesh`** controlled by custom GLSL vertex and fragment shaders with GPU-level backface culling.
* **Consequences:**
  * *Positive:* Reduces draw calls for 10,000 hotspots to **exactly 1 draw call**; frame rates locked at 60 FPS; VRAM usage $<50\text{ MB}$; smooth visual animations (pulsing rings, halo decay).
  * *Negative:* Requires maintaining custom GLSL shader code for lighting and pulsation physics.

---

### ADR-003: Deterministic & Statistical NLP over Generative LLMs

* **Status:** Accepted
* **Context:** Running 7B+ parameter LLMs locally consumes 4–6 GB of RAM, saturating our 8GB M1 machine, draining battery in $<3$ hours, and introducing hallucination risks. Paid commercial APIs (OpenAI) violate privacy, require credit cards, and fail the non-negotiable project principles.
* **Decision:** Use a lightweight, CPU-first deterministic NLP stack:
  1. Language Identification: FastText (`lid.176.ftz`, 917 KB).
  2. NER: spaCy (`en_core_web_sm` / `xx_ent_wiki_sm`, 12 MB).
  3. Topic Classification: Linear SGD classifier with TF-IDF on IPTC Media Topics.
  4. Summarization: Graph-based extractive TextRank (100% factual attribution, zero hallucination).
* **Consequences:**
  * *Positive:* Sub-millisecond latency; total NLP RAM footprint $<120\text{ MB}$; zero cloud costs; structurally immune to prompt injection attacks; 100% auditable sentence extraction.
  * *Negative:* Summaries are strictly extractive; cannot synthesize abstract high-level narratives across disparate events without predefined templates.

---

### ADR-004: MinHash/LSH + Incremental Online Centroid Story Clustering

* **Status:** Accepted
* **Context:** More than 60% of incoming global news items are syndicated duplicates or near-duplicate wire copies. Traditional batch clustering (HDBSCAN, K-Means) is too slow for real-time streaming, while naive pairwise cosine comparisons scale as $O(N^2)$.
* **Decision:** Deploy **DataSketch MinHash with LSH (32 bands, 4 rows)** to trap wire syndication ($J \ge 0.75$) in sub-linear time, combined with an **Incremental Online Centroid Clustering** algorithm over a rolling 72-hour sliding window.
* **Consequences:**
  * *Positive:* Prevents media volume inflation; updates story centroids in $<5\text{ ms}$; collapses duplicate wire copies into rich citation lists.
  * *Negative:* Requires careful tuning of Jaccard thresholds ($\tau = 0.65\text{ to }0.75$) to avoid over-clustering distinct regional events.

---

### ADR-005: SQLite 3 with FTS5 and BM25 Ranking as Primary Store

* **Status:** Accepted
* **Context:** The system requires a zero-configuration, robust local database capable of relational queries, spatial bounding-box filters, and sub-millisecond full-text keyword searches across 100,000+ articles.
* **Decision:** Standardize on **SQLite 3 with native FTS5 (Full-Text Search 5)** using BM25 relevance ranking and external content tables.
* **Consequences:**
  * *Positive:* Built-in on macOS (verified SQLite 3.43.2 with FTS5 active); zero background server overhead; sub-millisecond searches; single portable file (`globalnews.db`).
  * *Negative:* Concurrent write transactions require careful WAL (Write-Ahead Logging) mode configuration to prevent database locks during background ingestion.

---

### ADR-006: Tiered 3-Stage Storage Retention Hierarchy

* **Status:** Accepted
* **Context:** The host development environment has only 18 GiB of free disk space (92% used). Unbounded storage of news text would fill the disk within months.
* **Decision:** Implement a **3-tier hierarchical retention policy**:
  * Tier 1 (Raw HTML & XML): Auto-purged after 7 days.
  * Tier 2 (Extracted Article Metadata): Retained for 60 days.
  * Tier 3 (Consolidated Event Nodes): Retained permanently within user-configured storage quotas (e.g. 500 MB).
* **Consequences:**
  * *Positive:* Database size stabilizes at $<450\text{ MB}$ after one year of monitoring; total disk safety; preserves long-term intelligence trends without raw text bloat.
  * *Negative:* Users cannot view the full original HTML of articles older than 7 days (must click outbound source link).

---

### ADR-007: Multi-Source 5-Tier Ingestion Strategy

* **Status:** Accepted
* **Context:** Relying solely on GDELT introduces high false-positive rates and translation errors. Relying solely on Google News RSS is fragile due to bot-blocking and URL obfuscation.
* **Decision:** Ingest across 5 distinct tiers:
  1. Sensor Feeds (USGS, GDACS, NASA EONET).
  2. Institutional Outlets (UN, WHO, IAEA).
  3. Direct Wire RSS across 6 continents and 15+ languages.
  4. GDELT 2.0 (15-minute file streams for macro-validation).
  5. Scientific Feeds (arXiv, Nature, ScienceDaily).
* **Consequences:**
  * *Positive:* Eliminates single points of failure; provides ground-truth validation for physical disasters; neutralizes regional bias.
  * *Negative:* Ingestion pipeline must handle multiple payload schemas (GeoJSON, RSS 2.0, Atom, GKG CSV).

---

### ADR-008: Significance-Driven Scoring Decoupled from Media Volume

* **Status:** Accepted
* **Context:** Tabloid news and political controversies generate massive article volume, while critical scientific discoveries or regional natural disasters often have fewer articles. Equating volume with importance creates distorted intelligence.
* **Decision:** Formulate hotspot significance via multi-factor domain severity ($I_{\text{domain}}$), geographic reach ($R_{\text{geo}}$), and physical impact ($S_{\text{severity}}$), modulated by a **logarithmic volume dampener** $\Phi(N) = 1.0 + 0.15 \cdot \log_2(1 + N_{\text{sources}})$.
* **Consequences:**
  * *Positive:* Hotspots reflect genuine real-world consequence; major scientific breakthroughs are not eclipsed by celebrity drama.
  * *Negative:* Requires maintaining a domain severity weighting matrix that must be transparently documented and user-adjustable.

---

### ADR-009: Offline Two-Stage Toponym Resolution via Local GeoNames

* **Status:** Accepted
* **Context:** Extracting locations requires resolving ambiguous names (e.g. "Paris", "Georgia") without relying on external commercial geocoding APIs (Google Maps, Mapbox) which are paid and leak user queries.
* **Decision:** Deploy a **two-stage local resolver**:
  1. Administrative context extraction via spaCy co-occurring entities.
  2. Local SQLite lookup over GeoNames `cities15000` with population logarithmic weighting: $\text{Rank} = \text{CountryMatch} \times 100 + \log(\text{Population})$.
* **Consequences:**
  * *Positive:* 100% offline; executes in $<0.5\text{ ms}$; zero API fees; zero user data leakage.
  * *Negative:* Settlements with populations $<15,000$ are resolved to their parent administrative district or nearest major city.

---

### ADR-010: Trafilatura and DefusedXML for Secure Ingestion

* **Status:** Accepted
* **Context:** Parsing arbitrary untrusted XML and HTML feeds from the web introduces risks of XML External Entity (XXE) attacks, Billion Laughs denial-of-service, and boilerplate clutter (navigation links, cookie banners).
* **Decision:** Standardize on **`trafilatura`** for clean text and metadata extraction from HTML, and **`defusedxml`** for parsing all incoming XML/RSS payloads.
* **Consequences:**
  * *Positive:* Top-tier F1 precision in news extraction; strips advertising and cookie junk; guarantees immunity against XXE and entity expansion attacks.
  * *Negative:* Slightly higher CPU parsing time compared to raw regex (5–10 ms per page), mitigated by worker thread micro-batching.

