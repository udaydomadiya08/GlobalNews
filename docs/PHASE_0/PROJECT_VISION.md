# GLOBALNEWS — Product Vision & Design Principles

**Document ID:** `GN-DOC-P0-001`  
**Phase:** Phase 0 (Research & Feasibility)  
**Status:** Approved Architectural Baseline  
**Date of Inspection:** 2026-11-09  
**Target Hardware:** Apple Silicon M1 (8GB RAM), x86_64/ARM64 Consumer Desktops  

---

## 1. Executive Vision

**GLOBALNEWS** is a decentralized, local-first global intelligence platform that continuously monitors worldwide developments, detects consequential real-world events, evaluates their significance through transparent multi-dimensional criteria, and visualizes them on an interactive, realistic 3D Earth.

```
+-----------------------------------------------------------------------------------+
|                               GLOBALNEWS PLATFORM                                 |
|                                                                                   |
|   +--------------------+     +---------------------+     +--------------------+   |
|   |  Global Ingestion  | --> | Lightweight Engine  | --> | Interactive 3D     |   |
|   |  - Institutional   |     | - Deduplication     |     | Earth & Hotspots   |   |
|   |  - Wire Feeds      |     | - Toponym Geoparse  |     | - Multi-Spectral   |   |
|   |  - Disasters       |     | - Impact Scoring    |     | - Zoom LOD Res 3-7 |   |
|   |  - Science/Tech    |     | - Story Clustering  |     | - Event Panel      |   |
|   +--------------------+     +---------------------+     +--------------------+   |
|             |                           |                           |             |
|             +---------------------------+---------------------------+             |
|                                         v                                         |
|                          +-----------------------------+                          |
|                          | Local SQLite FTS5 Database  |                          |
|                          | Privacy-First / Zero Cloud  |                          |
|                          +-----------------------------+                          |
+-----------------------------------------------------------------------------------+
```

### 1.1 What GLOBALNEWS Is Not
To maintain strict architectural discipline, the platform is explicitly defined by what it rejects:
* **Not an ordinary RSS Reader:** It does not merely list chronologically sorted articles in an unranked feed.
* **Not a Clickbait News Aggregator:** It does not rank stories by user engagement, rage-baiting, virality, or advertising impressions.
* **Not a Superficial 3D Globe Demo:** The globe is a functional cartographic intelligence instrument, not a decorative Three.js toy.
* **Not a Heavyweight LLM Wrapper:** It does not pass raw article streams to expensive commercial cloud APIs (OpenAI, Anthropic) or require a high-end discrete GPU.

---

## 2. The 3D Interactive Earth Interface

The primary interface of GLOBALNEWS is an interactive, physically grounded 3D Earth. Planetary events are plotted at accurate geographic coordinates as dynamic, intelligent glowing hotspots.

```
       Planetary View (Res 3)          Continental View (Res 5)       Metropolitan View (Res 7)
          +---------------+               +---------------+               +---------------+
          |     .-"-.     |               |    / \__      |               |   [Event A]   |
          |   /  * *  \   |    Zoom In    |  /   *   \    |    Zoom In    |       |       |
          |  |   (*)   |  | ------------> | |   (*) * |   | ------------> |   [Hotspot]   |
          |   \   *   /   |               |  \_______/    |               |    /     \    |
          |     `-..-'    |               |               |               | [Src 1] [Src 2] |
          +---------------+               +---------------+               +---------------+
          Hotspot Clusters                Regional Outbreaks              Event Intelligence
```

### 2.1 Exploration Hierarchy
The user navigates through six continuous levels of detail (LOD):
1. **Planetary / World Level:** High-altitude vantage showing macroeconomic, geopolitical, climate, and major planetary trends. Hotspots are spatially clustered using Uber H3 hexagonal bins to prevent visual clutter.
2. **Continental Level:** Regional clusters resolve into multi-state or cross-border event dynamics.
3. **National Level:** Nation-state boundaries illuminate with administrative significance profiles and sovereign event vectors.
4. **Regional / Sub-national Level:** Provinces, states, and natural geographic features (rivers, fault lines, mountain ranges).
5. **Urban / Metropolitan Level:** Specific municipalities, ports, facilities, and epicenter coordinates.
6. **Individual Event Node:** Exact latitude/longitude point source with complete analytical context.

---

## 3. Intelligent Hotspot Visualization Physics

A hotspot's physical appearance on the globe is computed deterministically from objective event characteristics:

### 3.1 Visual Encoding Parameters
| Parameter | Mathematical Function | Semantic Meaning |
| :--- | :--- | :--- |
| **Radius ($R$)** | $R = R_{\text{base}} \cdot \log_2(1 + \text{GeographicReach})$ | Physical or geopolitical spread of impact. |
| **Glow Intensity ($I$)** | $I = I_{\text{base}} \cdot \text{SignificanceScore} \cdot C_{\text{confidence}}$ | Real-world importance modulated by evidence confidence. |
| **Pulse Frequency ($f$)** | $f = f_0 \cdot e^{-\lambda \Delta t_{\text{elapsed}}}$ | Velocity and urgency; breaking/developing events pulse rapidly. |
| **Temporal Decay ($\alpha$)** | $\alpha = \max(0.1, 1.0 - \gamma \Delta t_{\text{hours}})$ | Visual fade-out as events stabilize and move to historical status. |

### 3.2 Spectral Color Taxonomy
Color denotes the functional domain and directional impact of an event, avoiding arbitrary rainbow palettes:

```
+-------------------+---------------------+---------------------------------------------+
| Domain / Category | Hex / Spectral Band | Operational Meaning                         |
+-------------------+---------------------+---------------------------------------------+
| Disaster & Hazard | #E63946 (Crimson)   | Earthquakes, tsunamis, volcanic eruptions   |
| Armed Conflict    | #9B2226 (Dark Ruby) | Military engagements, kinetic hostilities   |
| Science & Tech    | #2A9D8F (Emerald)   | Discoveries, peer-reviewed breakthroughs    |
| Economy & Trade   | #F4A261 (Amber)     | Supply shocks, central bank shifts, tariffs |
| Public Health     | #9D4EDD (Purple)    | Outbreaks, epidemic surveillance            |
| Environment       | #457B9D (Azure)     | Wildfires, floods, atmospheric anomalies    |
| Developing/Uncert | #E9C46A (Cadmium)   | Rapidly emerging stories, single-source     |
| Neutral / Treaty  | #ADB5BD (Slate)     | Diplomatic summits, official ratifications  |
+-------------------+---------------------+---------------------------------------------+
```

### 3.3 Significance-Driven vs. Volume-Driven Hotspots
A foundational flaw in traditional news tracking is equating coverage volume with real-world importance. A celebrity legal dispute covered by 5,000 regional outlets will generate millions of words, while a major diplomatic treaty or breakthrough in room-temperature semiconductors might initially receive 12 wire reports.

GLOBALNEWS enforces **volume-decoupled significance scoring**:
$$\text{HotspotWeight} = \text{DomainSeverity} \times \text{GeographicReach} \times \text{EvidenceConfidence} \times \left(1 + \kappa \cdot \log_{10}(N_{\text{independent\_sources}})\right)$$
Where $\kappa \approx 0.15$. A $100\times$ increase in article volume only produces a $1.3\times$ increase in visual weight, preventing syndicated spam from eclipsing consequential developments.

---

## 4. The Event Intelligence Panel

Selecting any hotspot triggers a slide-out or docked analytical dossier structured into eight verifiable sections:

```
+-----------------------------------------------------------------------------------+
| EVENT INTELLIGENCE DOSSIER                                                  [X]   |
+-----------------------------------------------------------------------------------+
| TITLE: Mw 7.2 Off-Coast Seismic Event & Regional Tsunami Advisory                 |
| LOCATION: Miyagi Prefecture, Japan [38.298° N, 142.373° E] (Confidence: 98%)     |
| STATUS: CONFIRMED | IMPACT: HIGH | TEMPORAL: 14m ago (Active Alert)               |
+-----------------------------------------------------------------------------------+
| 1. SYNTHESIS SUMMARY (Extractive, 3 Sentences):                                   |
|    - A magnitude 7.2 earthquake occurred at depth 45km off Honshu coast.          |
|    - JMA issued a 1-meter coastal tsunami advisory for Miyagi and Iwate.          |
|    - Shinkansen high-speed rail automatically halted; no major damage reported.   |
|                                                                                   |
| 2. IMPACT MATRIX:                                                                 |
|    - Human Safety: Moderate Risk | Infrastructure: Railway disruption             |
|    - Economic Impact: Localized   | Environmental Impact: Tsunami swell 0.3m      |
|                                                                                   |
| 3. EVIDENCE & CORROBORATION:                                                      |
|    - Confidence: 99% (Institutional: USGS, JMA, GDACS + 4 Independent Wires)      |
|    - Diversity Index: 0.94 (Cross-jurisdictional verification)                     |
|                                                                                   |
| 4. CHRONOLOGICAL TIMELINE:                                                        |
|    - 11:24 UTC: USGS P-wave trigger detected (M7.1 preliminary)                   |
|    - 11:27 UTC: JMA issues Coastal Advisory                                       |
|    - 11:35 UTC: Reuters confirms bullet train suspension                          |
|                                                                                   |
| 5. PRIMARY CITATIONS & ATTRIBUTION:                                               |
|    - [USGS Earthquake Alert] https://earthquake.usgs.gov/fdsnws/...               |
|    - [GDACS Report #1042] https://www.gdacs.org/report.aspx?id=1042...            |
|    - [NHK World Dispatch] https://www3.nhk.or.jp/nhkworld/en/news/...            |
+-----------------------------------------------------------------------------------+
```

---

## 5. Non-Negotiable Project Principles

Every engineering decision across all phases must adhere to these 15 mandates:

1. **Research Before Implementation:** No code is merged without verified documentation, architectural evaluation, and tested failure modes.
2. **Reuse Before Reinventing:** Adopt mature, battle-tested open-source libraries (Three.js, SQLite FTS5, spaCy, Trafilatura) instead of writing custom bespoke infrastructure.
3. **Free and Open-Source-First:** The platform core must be completely functional using permissive, copyleft, or open-data infrastructure.
4. **No Mandatory Paid APIs:** Core operation must not depend on OpenAI, NewsAPI, Bing, Google Maps, or any metered API key.
5. **No Mandatory Discrete GPU:** The intelligence engine and 3D globe must run smoothly at 60 FPS on integrated graphics (e.g. Apple M1 7/8-core GPU).
6. **No Mandatory Heavyweight LLMs:** No 7B+ parameter local LLMs consuming 6GB of RAM. Processing relies on deterministic rules, statistical tokenizers, and fast extractive algorithms.
7. **CPU-First Lightweight Processing:** All batch and streaming NLP must execute on standard CPU threads within constrained memory budgets ($< 350\text{ MB}$ total application footprint).
8. **Local-First Architecture:** Data is ingested, indexed, and stored directly on the user's workstation. If the network goes down, the platform remains fully explorable.
9. **User-Controlled Storage:** Hard disk quotas are user-configurable with strict automated tiered data pruning.
10. **Transparent and Explainable Intelligence:** Every score, rating, cluster, and classification must expose its mathematical derivation and input variables.
11. **Accurate Source Attribution:** No synthetic content generation without source links. Every claim links to original publisher URLs.
12. **Honest Uncertainty & Confidence:** Single-source or disputed reports are explicitly labeled as `UNVERIFIED` or `CONTESTED`. Uncertainty is never smoothed over.
13. **Global & Multilingual Ambitions:** System architecture must support non-Latin scripts (Arabic, Cyrillic, CJK, Devanagari) and native translation mapping.
14. **Security & Privacy by Design:** Zero tracking, zero telemetry, full SSRF immunity during feed ingestion, and zero prompt injection attack surfaces.
15. **No Artificial Feature Inflation:** Focus on delivering high-integrity news intelligence before adding secondary experimental bells and whistles.

