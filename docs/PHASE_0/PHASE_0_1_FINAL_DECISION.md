# GLOBALNEWS — Phase 0.1 Formal Decision & Implementation Readiness

**Document ID:** `GN-DOC-P01-004`  
**Phase:** Phase 0.1 (Architecture Audit & Claim Verification)  
**Status:** Formal Engineering Decision Document  
**Date of Decision:** 2026-11-09  

---

## 1. Formal Readiness Decision

### DECISION: **CONDITIONALLY READY FOR PHASE 1**

The architectural foundations of **PROJECT GLOBALNEWS** are viable, robust, and mathematically sound. However, proceeding directly to implementation without incorporating the audit findings would have introduced fatal vulnerabilities (DNS rebinding on localhost, fragmentation of long-running wars via the 72-hour clustering window, and immediate daemon memory overrun).

Implementation of Phase 1 is **approved conditionally upon strict adherence to the Phase 0.1 Corrected Architecture**.

```
+---------------------------------------------------------------------------------------------------+
|                               PHASE 0.1 READINESS SCORECARD                                       |
+------------------------------------+---------------+----------------------------------------------+
| Engineering Pillar                 | Status        | Condition for Phase 1                        |
+------------------------------------+---------------+----------------------------------------------+
| 1. System Hardware Feasibility     | PASS          | Revised daemon RAM budget to <280 MB.        |
| 2. Storage & Disk Safety (18GB)    | PASS          | Enforce 3-tier LRU retention engine.         |
| 3. Localhost Security Shield       | CONDITIONAL   | Must implement Token Auth & Host Validation. |
| 4. Database Concurrency & FTS5     | PASS          | Single-writer queue for WAL mode.            |
| 5. Long-Term Story Tracking        | CONDITIONAL   | Must deploy Two-Tier (Macro/Micro) model.    |
| 6. Cross-Lingual Ingestion         | CONDITIONAL   | Must use GeoNames ID entity graph alignment. |
| 7. Multi-Source Coverage           | PASS          | Added UN OCHA ReliefWeb to Tier 2 sources.   |
| 8. 3D Globe Rendering Physics      | PASS (Phase 2)| Raycasting constrained to H3 clusters.       |
+------------------------------------+---------------+----------------------------------------------+
```

---

## 2. Mandatory Conditions for Phase 1 Implementation

Before merging code in Phase 1, the implementation must satisfy these four mandatory conditions:

1. **Condition C-01: Zero-Trust Localhost Security Shield**  
   The HTTP/SSE daemon (`gn-core`) must not accept any connection without validating:
   * That the `Host` header strictly matches `127.0.0.1:<port>` or `localhost:<port>`.
   * That the request contains a valid 256-bit ephemeral Bearer token matching `~/.globalnews/auth.token`.
   * That cross-origin requests originate strictly from whitelisted origins.

2. **Condition C-02: Two-Tier Event Relational Schema**  
   The SQLite database schema must define both `macro_events` (persistent containers for multi-month wars, economic crises, treaties) and `micro_events` (localized 72-hour incidents linking to parents).

3. **Condition C-03: Single Dedicated SQLite Writer Queue**  
   All SQLite database mutations must pass through an asynchronous FIFO queue (`asyncio.Queue`) handled by a single dedicated writer thread, preventing database lock contention in WAL mode.

4. **Condition C-04: Parser-Disabled spaCy Extraction**  
   To honor the revised $280\text{ MB}$ memory budget and sub-15ms latency, spaCy pipelines must be instantiated with heavy syntactic parsers explicitly disabled (`disable=["parser", "attribute_ruler", "lemmatizer"]`).

---

## 3. Exact Scope of Phase 1: Ingestion, Security & Data Foundation

Phase 1 will construct the foundational data engine (`gn-core`). In strict accordance with phase boundaries, **Phase 1 will not build the 3D globe visualization interface** (scheduled for Phase 2).

### 3.1 Phase 1 Milestones & Deliverables

```
+-----------------------------------------------------------------------------------+
| PHASE 1 WORK BREAKDOWN STRUCTURE                                                  |
+-----------------------------------------------------------------------------------+
| Milestone 1.1: Project Scaffolding & Configuration                                |
| - Dynamic port discovery (7474-7484) and daemon.json registration                 |
| - Ephemeral 256-bit auth token generation and 0600 file security                  |
| - Host header and CORS origin validation middleware                               |
+-----------------------------------------------------------------------------------+
| Milestone 1.2: SQLite Database & Migration Engine                                 |
| - SQLite 3 initialization with PRAGMA journal_mode=WAL and busy_timeout=5000     |
| - Two-Tier schema: macro_events, micro_events, articles, sources, citations       |
| - FTS5 virtual tables with BM25 ranking and trigram/porter tokenizers             |
| - Asynchronous single-writer queue worker                                         |
+-----------------------------------------------------------------------------------+
| Milestone 1.3: Security Ingestion Layer                                           |
| - Pre-flight DNS resolver blocking RFC 1918 private subnets and loopbacks (SSRF)  |
| - defusedxml parser preventing XML External Entity (XXE) and Billion Laughs       |
| - Response body size ceiling (max 5 MB per feed)                                  |
+-----------------------------------------------------------------------------------+
| Milestone 1.4: Multi-Tier Feed Ingestors                                          |
| - Tier 1: USGS Earthquakes real-time GeoJSON collector (60s cache cycle)          |
| - Tier 1: GDACS Disasters GeoJSON/RSS collector (15m cache cycle)                |
| - Tier 1: NASA EONET v3 Natural Events collector                                  |
| - Tier 2: UN News & UN OCHA ReliefWeb REST/RSS collectors                         |
| - Tier 3: Polymorphic Wire RSS collector with ETag / If-Modified-Since            |
+-----------------------------------------------------------------------------------+
| Milestone 1.5: MinHash/LSH Deduplication & Two-Tier Clustering                    |
| - MinHash (128 permutations, 32 bands, 4 rows) for wire-syndication detection    |
| - Micro-event clustering (72-hour sliding window)                                 |
| - Macro-event linking via GeoNames IDs and IPTC topic codes                       |
+-----------------------------------------------------------------------------------+
| Milestone 1.6: 3-Tier Storage Retention Engine                                    |
| - Automated daily purge of Tier 1 raw HTML older than 7 days                      |
| - Automated purge of unpinned Tier 2 metadata older than 60 days                  |
| - Quota monitor triggering VACUUM when database exceeds 500 MB limit              |
+-----------------------------------------------------------------------------------+
| Milestone 1.7: Local REST API & Resilient SSE Event Stream                        |
| - GET /api/v1/events (Spatial bounding-box and significance filters)              |
| - GET /api/v1/search (Sub-2ms FTS5 BM25 search)                                   |
| - GET /api/v1/stream (SSE stream with Last-Event-ID missed event ring buffer)     |
+-----------------------------------------------------------------------------------+
```

---

## 4. Phase 1 Acceptance Criteria & Verification Tests

Phase 1 will be considered complete only when every test in this verification suite passes on the host Apple Silicon M1 machine:

| Test ID | Test Description | Target Success Metric |
| :--- | :--- | :--- |
| **TEST-SEC-01** | DNS Rebinding Simulation | Requests with `Host: attacker.com` return `403 Forbidden`. |
| **TEST-SEC-02** | Token Authentication | Requests without valid Bearer token return `401 Unauthorized`. |
| **TEST-SEC-03** | SSRF Rejection | Feed URLs resolving to `127.0.0.1` or `192.168.1.1` are blocked before TCP connect. |
| **TEST-SEC-04** | XXE Bomb Defense | Malicious XML with recursive entity expansion triggers `defusedxml.DefusedXmlException`. |
| **TEST-DB-01** | FTS5 Search Latency | 100 BM25 search queries over 10,000 indexed articles average $< 3.0\text{ ms}$. |
| **TEST-DB-02** | Concurrent Write Safety | 50 concurrent async write tasks produce zero `database is locked` errors. |
| **TEST-ING-01** | USGS Earthquake Ingest | Successfully ingests live earthquakes and extracts exact coordinates. |
| **TEST-ING-02** | ReliefWeb Ingestion | Successfully fetches and parses UN OCHA crisis situation reports. |
| **TEST-DED-01** | Wire Syndication Filter | Near-duplicate AP articles ($J \ge 0.75$) merge into a single micro-event citation. |
| **TEST-RET-01** | Storage Retention Pruning| Simulated 8-day-old raw records are cleanly removed; database size shrinks. |
| **TEST-SSE-01** | Missed-Event Recovery | Client reconnecting with `Last-Event-ID` receives all missed events from ring buffer. |
| **TEST-MEM-01** | Daemon RAM Ceiling | Idle resident memory of running daemon remains strictly $< 280\text{ MB}$ RSS. |

---

## 5. Phase 1 Dependency Tree

To adhere strictly to zero-heavyweight-dependencies, Phase 1 requires only:
* **Python Runtime:** Python 3.13 (Verified installed).
* **Core Libraries:**
  * `aiohttp` (Verified installed, v3.11.14 — HTTP/SSE server and async client).
  * `defusedxml` (Verified installed, v0.7.1 — XXE-safe XML parsing).
  * `beautifulsoup4` (Verified installed, v4.12.3 — HTML sanitization).
  * `sqlite3` (Verified installed, v3.43.2 with FTS5).
  * Optional: `spaCy` (Verified installed, v3.8.7, with parser disabled).

Zero commercial cloud dependencies. Zero API keys. Zero external network proxies.

---

## 6. Implementation Boundaries Reminder

* **Do NOT build the 3D globe visualization in Phase 1.** (Scheduled for Phase 2).
* **Do NOT install 7B+ parameter LLMs.**
* **Do NOT download massive multi-gigabyte datasets.**
* **Do NOT violate the $280\text{ MB}$ daemon memory ceiling.**

---

*This document concludes Phase 0.1. Awaiting explicit user instruction to proceed to Phase 1 implementation.*

