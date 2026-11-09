# GLOBALNEWS — Storage & Resource Management Model

**Document ID:** `GN-DOC-P0-008`  
**Phase:** Phase 0 (Research & Feasibility)  
**Status:** Approved Architectural Baseline  
**Inspection Date:** 2026-11-09  
**Hardware Baseline:** Apple Silicon M1 (8GB Unified RAM, 18 GiB Available Disk Space on APFS).

---

## 1. Physical Hardware Constraints Analysis

Inspection of the operational development environment establishes strict physical operational ceilings:
* **Host RAM:** $8,589,934,592\text{ bytes}$ ($8.00\text{ GB}$). macOS system processes and user applications typically consume $5.5\text{ to }6.5\text{ GB}$. The allowable RAM budget for GLOBALNEWS must not exceed **$350\text{ MB}$ total** ($<4.3\%$ of physical RAM) to prevent memory pressure warnings and APFS swap file creation.
* **Available Disk Storage:** Current available space is **$18\text{ GiB}$** (Filesystem: `/dev/disk3s5`, capacity used: $92\%$). Under no circumstances may GLOBALNEWS accumulate unbounded news archives, uncontrolled caches, or unpruned text dumps.
* **CPU & Thermal Envelope:** Apple M1 fanless MacBook Air. Heavy continuous multi-core processing triggers thermal throttling. Ingestion and NLP tasks must execute in low-priority bursts.

---

## 2. Global Memory Budget Allocation

The total allocated memory footprint is budgeted across the decoupled components:

```
+-----------------------------------------------------------------------------------+
|                        TOTAL SYSTEM MEMORY BUDGET: 330 MB                         |
+-----------------------------------------------------------------------------------+
                                          |
             +----------------------------+----------------------------+
             |                                                         |
             v                                                         v
+-----------------------------------+     +-----------------------------------+
| THE INTELLIGENCE DAEMON           |     | THE CLIENT INTERFACE (WEBVIEW)    |
| Budget: 180 MB Maximum            |     | Budget: 150 MB Maximum            |
+-----------------------------------+     +-----------------------------------+
| - Runtime Base (Node/Python): 40MB|     | - WKWebView Base Engine: 45 MB    |
| - FastText Model & Buffer:    5 MB|     | - Three.js WebGL VRAM Buffers:45MB|
| - spaCy Small NER Model:     40 MB|     | - GeoJSON Vector Geometry:  15 MB |
| - Active MinHash LSH Index:  25 MB|     | - UI DOM & Dossier Drawer:  20 MB |
| - SQLite In-Memory Cache:    35 MB|     | - Headroom / GC Margin:     25 MB |
| - Headroom / GC Margin:      35 MB|     +-----------------------------------+
+-----------------------------------+
```

---

## 3. Tiered Storage Hierarchy & Data Pruning Architecture

To prevent disk saturation while preserving rich historical intelligence, data moves through a **3-tier lifecycle hierarchy**:

```
[ Raw RSS / Web Ingestion ]
             |
             v
+-----------------------------------------------------------------------------------+
| TIER 1: TEMPORARY RAW ARTICLE CACHE                                               |
| - Full raw HTML, raw XML feeds, intermediate parsing trees                        |
| - Retention: 7 Days (Strict sliding window)                                       |
| - Pruning Policy: Automatically purged daily via SQLite DELETE                    |
+-----------------------------------------------------------------------------------+
             | (Extract clean entities, coordinates, summary)
             v
+-----------------------------------------------------------------------------------+
| TIER 2: EXTRACTED DOCUMENT METADATA                                               |
| - Clean article title, 3-sentence summary, publication date, source URL, hash     |
| - Retention: 60 Days                                                              |
| - Pruning Policy: Preserved for 60 days; articles linked to bookmarked events     |
|   are permanently preserved.                                                      |
+-----------------------------------------------------------------------------------+
             | (Consolidate into semantic event nodes)
             v
+-----------------------------------------------------------------------------------+
| TIER 3: CONSOLIDATED EVENT INTELLIGENCE NODES                                     |
| - Event Title, Centroid Coordinates, IPTC Category, Peak Significance Score,      |
|   Multi-factor Impact Matrix, Chronological Timeline, Source Citations           |
| - Retention: Permanent (or user-configured quota limit)                           |
| - Average Node Size: 1.2 KB per event                                             |
+-----------------------------------------------------------------------------------+
```

---

## 4. Realistic Quantitative Storage Projections

Calculations are based on a realistic worldwide daily intake of **5,000 articles/day** ($150,000\text{ articles/month}$):

```
+-----------------------------------------------------------------------------------+
|                  EMPIRICAL STORAGE CALCULATIONS (MONTHLY BASIS)                   |
+--------------------------+--------------------+------------------+----------------+
| Data Category            | Raw Unit Size      | Monthly Volume   | Effective Size |
+--------------------------+--------------------+------------------+----------------+
| Raw HTML Payloads (T1)   | 85 KB / article    | 150,000 items    | Expired (0 MB) |
| Extracted Metadata (T2)  | 1.8 KB / article   | 150,000 items    | 270 MB (60d)   |
| Event Clusters (T3)      | 1.2 KB / event     | 4,500 events     | 5.4 MB / month |
| SQLite FTS5 Index        | ~25% of text       | -                | ~70 MB / month |
| Spatial R*Tree Index     | 16 bytes / record  | 4,500 events     | < 1 MB         |
+--------------------------+--------------------+------------------+----------------+
```

### Cumulative Annual Storage Projections
* **With Default Retention (7-day Tier 1 purge, 60-day Tier 2 purge, permanent Tier 3 events):**
  * Database file after 1 Year: **$\approx 380\text{ MB}$ to $450\text{ MB}$**.
* **Storage Footprint vs. Disk Capacity:** $450\text{ MB}$ represents only **$2.4\%$ of available disk space ($18\text{ GiB}$)**, ensuring complete safety against disk exhaustion.

---

## 5. User-Controlled Storage Quotas & Eviction Engine

GLOBALNEWS provides explicit, user-configured storage caps selectable directly from the interface:

```
+-----------------------------------------------------------------------------------+
| STORAGE CONFIGURATION PANEL                                                       |
+-----------------------------------------------------------------------------------+
| [x] Automatic Quota Enforcement                                                   |
| Storage Limit: ( ) 250 MB   (*) 500 MB   ( ) 1.0 GB   ( ) 2.0 GB                  |
| Current Database Size: 142.4 MB (28% of 500 MB budget)                            |
|                                                                                   |
| RETENTION POLICY:                                                                 |
| Raw HTML Cache:      [ 7 Days   v ]                                               |
| Article Metadata:    [ 60 Days  v ]                                               |
| Historical Events:   [ Permanent (Respect Quota) v ]                              |
|                                                                                   |
| [ Clear Temporary Cache Now ]       [ Export Database to SQLite ]                 |
+-----------------------------------------------------------------------------------+
```

### Automated LRU Eviction Logic
When database size reaches $90\%$ of the user-configured ceiling:
1. Trigger `VACUUM` and purge all Tier 1 raw records older than 48 hours.
2. Purge Tier 2 article records with zero user bookmarks starting from the oldest.
3. Consolidate low-significance historical events ($\text{Significance} < 3.0$) into monthly regional summary records.
4. User-bookmarked events and pinned dossiers are **strictly immune from eviction**.

---

## 6. CPU, Battery & Thermal-Aware Throttling

To prevent background battery drain on laptops running on battery power:

### 6.1 Power State Detection
The daemon monitors host power status via macOS `pmset` or OS battery APIs:
* **AC Power Connected (Mains):**
  * Ingestion Interval: Every 5 minutes.
  * Ingestion Batch Size: 50 articles per batch.
  * Parallel Threads: 2 worker threads.
* **Battery Power Active:**
  * Ingestion Interval: Every 30 minutes.
  * Ingestion Batch Size: 20 articles per batch.
  * Parallel Threads: 1 worker thread (pinned to Efficiency cores via OS scheduling).
* **Low Battery (< 20%):**
  * Ingestion Suspended. Intelligence engine switches to read-only local search and exploration until AC power is restored.

### 6.2 Process Priority & CPU Throttling
All background NLP and clustering workers are launched with maximum POSIX niceness:
```bash
nice -n 19 python3 -m gn_core.worker
```
Between processing each batch of 20 articles, the worker thread yields execution via an explicit $250\text{ ms}$ sleep interval, allowing OS CPU scheduling to prioritize interactive user applications and prevent thermal buildup.

