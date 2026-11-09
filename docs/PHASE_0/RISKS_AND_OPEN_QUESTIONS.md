# GLOBALNEWS — Risk Register & Open Technical Questions

**Document ID:** `GN-DOC-P0-011`  
**Phase:** Phase 0 (Research & Feasibility)  
**Status:** Approved Architectural Baseline  
**Inspection Date:** 2026-11-09  

---

## 1. Comprehensive Risk Register

Each risk is evaluated across **Probability (P)**, **Impact (I)**, and assigned a **Risk Priority Number (RPN = P × I)** on a 1–5 scale, alongside concrete mitigation strategies:

```
+---------------------------------------------------------------------------------------------------+
|                                     PROJECT RISK MATRIX                                           |
+------+--------------------------+---+---+-----+---------------------------------------------------+
| ID   | Risk Description         | P | I | RPN | Mitigation Strategy                               |
+------+--------------------------+---+---+-----+---------------------------------------------------+
| R-01 | Feed Bit Rot & Schema    | 4 | 3 | 12  | Polymorphic feed ingestion pipeline; fallback to  |
|      | Format Drift             |   |   |     | direct HTML scraping via Trafilatura if RSS breaks|
+------+--------------------------+---+---+-----+---------------------------------------------------+
| R-02 | WebGL Context Loss on    | 3 | 4 | 12  | Listen to `webglcontextlost` and restore scene    |
|      | macOS Sleep / Wake       |   |   |     | graph from cached InstancedBuffer attributes.     |
+------+--------------------------+---+---+-----+---------------------------------------------------+
| R-03 | SQLite Database Lock     | 3 | 4 | 12  | Enforce WAL (Write-Ahead Logging) mode, busy      |
|      | Contention Under Writes  |   |   |     | timeout = 5000ms, dedicated single-writer thread. |
+------+--------------------------+---+---+-----+---------------------------------------------------+
| R-04 | Toponym Ambiguity in     | 4 | 3 | 12  | Two-stage resolution with administrative context  |
|      | Geoparsing               |   |   |     | scoring and logarithmic population prior.         |
+------+--------------------------+---+---+-----+---------------------------------------------------+
| R-05 | Breaking News Volume     | 3 | 3 |  9  | Micro-batching queues; rate-limiting intake;      |
|      | Ingestion Spike          |   |   |     | immediate drop of non-essential Tier 1 payloads.  |
+------+--------------------------+---+---+-----+---------------------------------------------------+
| R-06 | Coordinated Bot Network  | 2 | 4 |  8  | Cross-corroboration gating; single-source events  |
|      | Disinformation Campaign |   |   |     | strictly capped at `EMERGING / UNVERIFIED` status.|
+------+--------------------------+---+---+-----+---------------------------------------------------+
| R-07 | Aggregator Copyright /   | 2 | 4 |  8  | Strict Fair Use compliance; extractive summaries  |
|      | Publisher Cease & Desist |   |   |     | only; no full-text republication; robots.txt honor|
+------+--------------------------+---+---+-----+---------------------------------------------------+
| R-08 | Disk Space Exhaustion on | 2 | 5 | 10  | Automated 3-tier retention engine; hard quota cap |
|      | Constrained Drive (18GB) |   |   |     | with automated LRU vacuuming when space < 10%.   |
+------+--------------------------+---+---+-----+---------------------------------------------------+
```

---

## 2. In-Depth Risk Analysis & Mitigation Engineering

### 2.1 Technical Risk: WebGL Context Loss on macOS Sleep/Wake Cycles (R-02)
* **Failure Mechanism:** On Apple Silicon MacBooks, when the lid closes or the system enters deep sleep, the macOS WindowServer terminates active Metal/WebGL contexts to conserve DRAM power. Upon waking, the canvas becomes completely black.
* **Mitigation Architecture:**
  1. Register event listeners for `webglcontextlost` and `webglcontextrestored` on the canvas element.
  2. Maintain a serialized in-memory snapshot of current event cluster positions and active H3 cells.
  3. Upon context restoration, re-compile custom GLSL shaders, re-bind instanced buffer attributes, and re-populate the scene graph without requiring a full page refresh.

### 2.2 Operational Risk: SQLite Lock Contention Under Concurrent Write/Read (R-03)
* **Failure Mechanism:** The background daemon writes batches of parsed articles and clusters into SQLite while the user concurrently executes sub-millisecond FTS5 search queries from the 3D interface, leading to `sqlite3.OperationalError: database is locked`.
* **Mitigation Architecture:**
  1. Enable Write-Ahead Logging (`PRAGMA journal_mode = WAL;`).
  2. Set synchronous mode to normal (`PRAGMA synchronous = NORMAL;`).
  3. Configure busy timeout to 5,000 milliseconds (`PRAGMA busy_timeout = 5000;`).
  4. Design the backend so that **all write operations are serialized through a single dedicated write worker queue**, while read operations utilize non-blocking concurrent connections.

### 2.3 Data Integrity Risk: Toponym Ambiguity & False Geoparsing (R-04)
* **Failure Mechanism:** An article mentions *"President signs trade agreement with Jordan"* and the geoparser places a hotspot in Jordan, Minnesota (USA) instead of the Hashemite Kingdom of Jordan.
* **Mitigation Architecture:**
  1. Contextual Co-occurrence: Boost country scores when co-occurring national capital or leader entities appear in the text (e.g. "Amman" or "King Abdullah").
  2. Feature Class Weighting: Prioritize sovereign country features (`PCLI`) and major metropolitan capitals over small populated places (`PPL`).
  3. Minimum Confidence Gate: If toponym confidence score falls below $0.60$, map the event to the national centroid rather than an arbitrary city coordinate, and flag the location with an uncertainty circle.

---

## 3. Open Technical Questions for Phase 1 Validation

These questions cannot be resolved purely through documentation and require empirical testing during Phase 1:

### Question 1: Cross-Lingual Geoparsing Precision on Non-Latin Scripts
* **Hypothesis:** spaCy's `xx_ent_wiki_sm` combined with GeoNames multilingual alternate names (`alternateNames.txt`) can accurately resolve Arabic, Cyrillic, and Chinese toponyms to standard WGS84 coordinates without pre-translation into English.
* **Phase 1 Validation Plan:** Ingest a test corpus of 200 non-English news articles (Arabic Al Jazeera, French France24, Russian TASS, Japanese NHK) and measure geoparsing accuracy against human-annotated ground truth.

### Question 2: Optimal MinHash Shingle Size for Cross-Publisher Syndication
* **Hypothesis:** 3-word shingles provide higher recall for syndicated news variations (where editors alter the introductory lead paragraph), while 4-word shingles reduce false-positive collisions across unrelated articles discussing the same generic topic.
* **Phase 1 Validation Plan:** Run a synthetic perturbation benchmark across 1,000 Associated Press wire articles subjected to simulated editorial rewrites (10% to 30% sentence deletion and substitution) to determine the exact optimal Jaccard threshold ($\tau$) and shingle size ($n$).

### Question 3: 24-Hour macOS M1 Battery Drain Under Continuous Daemon Execution
* **Hypothesis:** Operating with POSIX `nice = 19`, 30-minute battery polling intervals, and efficiency-core pinning will consume $< 3\%$ of total battery capacity over a continuous 24-hour sleep/wake monitoring cycle.
* **Phase 1 Validation Plan:** Deploy the headless ingestion prototype on the physical Apple M1 machine, disconnect AC power, and log battery percentage and CPU energy usage via `powermetrics` over a 24-hour period.

