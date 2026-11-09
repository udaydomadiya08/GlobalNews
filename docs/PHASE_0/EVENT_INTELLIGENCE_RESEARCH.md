# GLOBALNEWS — Event Intelligence, Clustering & Evolution Research

**Document ID:** `GN-DOC-P0-005`  
**Phase:** Phase 0 (Research & Feasibility)  
**Status:** Approved Architectural Baseline  
**Inspection Date:** 2026-11-09  
**Core Objective:** Transforming noisy, fragmented global news streams into coherent, evolving event graphs with transparent significance scoring.

---

## 1. The Streaming Event Ingestion Challenge

In a global monitoring environment, thousands of articles enter the platform continuously:
* 60% are near-duplicate wire syndications (e.g. Associated Press or Reuters reports republished across dozens of regional news outlets).
* 25% are incremental updates or follow-up reactions to existing breaking events.
* 10% are routine non-event background commentary or feature articles.
* 5% represent distinct, new real-world breaking developments.

Without intelligent deduplication, clustering, and lifecycle modeling, any 3D visualization degrades into a sea of overlapping redundant markers.

```
                      INCOMING RAW ARTICLES
                                |
                                v
               +---------------------------------+
               | 1. MinHash / LSH Shingle Filter |
               +---------------------------------+
                                |
             +------------------+------------------+
             | Is near-duplicate (Jaccard >= 0.75)?|
             +------------------+------------------+
                    /                        \
                  YES                         NO
                  /                            \
                 v                              v
    +--------------------------+  +-------------------------------+
    | Collapse into Citation   |  | 2. Incremental Story Cluster  |
    | Record (Increment Source |  | (Match against active events) |
    | Count & Diversity)       |  +-------------------------------+
    +--------------------------+                |
                                     +----------+----------+
                                     | Match Active Event? |
                                     +----------+----------+
                                            /        \
                                          YES         NO
                                          /            \
                                         v              v
                           +------------------+   +-------------------+
                           | Evolve Existing  |   | Initialize New    |
                           | Event (Timeline, |   | Event Cluster     |
                           | Confidence, Geo) |   | (Status: Emerging)|
                           +------------------+   +-------------------+
```

---

## 2. Near-Duplicate & Wire Syndication Detection

### 2.1 The MinHash & LSH Mechanism
* **Token Shingling:** Extract 3-gram character/word shingles from the normalized article body:
  $$S(D) = \{ (w_1, w_2, w_3), (w_2, w_3, w_4), \dots, (w_{n-2}, w_{n-1}, w_n) \}$$
* **MinHash Signatures:** Using $k = 128$ independent hash functions $h_i(x) = (a_i \cdot x + b_i) \pmod p$, compute the minimum hash value for each document to create a 128-integer signature vector $\vec{sig}(D)$.
* **Locality-Sensitive Hashing (LSH) Partitioning:** Partition the 128 hashes into $b = 32$ bands with $r = 4$ rows per band ($b \cdot r = 128$).
* **Candidate Collision Probability:**
  $$P(\text{Candidate Collision}) = 1 - (1 - s^4)^{32}$$
  At a Jaccard similarity threshold $s = 0.75$, the probability of two syndicated articles being flagged as candidates is $>98.7\%$, while documents with $s < 0.20$ have a collision probability $<0.01\%$.
* **Action:** When a document matches an existing article with $s \ge 0.75$, it is tagged as a **Syndicated Duplicate**. It is not plotted as a new event, but added to the primary article's source citation list, updating source diversity metrics.

---

## 3. Streaming Story Clustering & Centroid Tracking

### 3.1 Temporal Sliding Window
Real-world events are bounded in time. Story clustering operates across a **rolling 72-hour active memory buffer**. Events older than 72 hours without updates are transitioned into the historical SQLite archive, freeing RAM.

### 3.2 Multi-Modal Cluster Similarity Metric
An incoming non-duplicate article $\vec{d}_i$ is compared to all active event clusters $C_k = \{ \vec{\mu}_k, \mathcal{E}_k, (\phi_k, \lambda_k), t_k \}$ using a weighted multi-modal distance function:

$$\text{Similarity}(\vec{d}_i, C_k) = w_{\text{text}} \cdot S_{\text{cosine}}(\vec{d}_i, \vec{\mu}_k) + w_{\text{ent}} \cdot J(\mathcal{E}_i, \mathcal{E}_k) + w_{\text{geo}} \cdot G(d_i, C_k) + w_{\text{time}} \cdot T(\Delta t)$$

Where:
1. **Lexical Cosine Similarity ($S_{\text{cosine}}$):** Cosine angle between article TF-IDF vector $\vec{d}_i$ and the running cluster centroid $\vec{\mu}_k$. Weight: $w_{\text{text}} = 0.40$.
2. **Named Entity Jaccard Overlap ($J$):** Overlap of recognized organizations, persons, and geopolitical entities:
   $$J(\mathcal{E}_i, \mathcal{E}_k) = \frac{|\mathcal{E}_i \cap \mathcal{E}_k|}{|\mathcal{E}_i \cup \mathcal{E}_k|}$$
   Weight: $w_{\text{ent}} = 0.30$.
3. **Geographic Proximity Gate ($G$):** Haversine distance $D_{\text{km}}$ between resolved coordinates:
   $$G(d_i, C_k) = \max\left(0, 1.0 - \frac{D_{\text{km}}}{250\text{ km}}\right)$$
   Weight: $w_{\text{geo}} = 0.20$. If distance $> 500\text{ km}$, this term acts as an inhibitory filter (similarity penalization).
4. **Temporal Recency Decay ($T$):**
   $$T(\Delta t) = e^{-\lambda \Delta t}$$
   where $\Delta t$ is hours since the cluster's latest update ($\lambda \approx 0.05$). Weight: $w_{\text{time}} = 0.10$.

### 3.3 Cluster Centroid Update
If $\max_k \text{Similarity}(\vec{d}_i, C_k) \ge \tau_{\text{cluster}}$ (default $\tau = 0.65$), article $\vec{d}_i$ merges into cluster $C_k$:
$$\vec{\mu}_k^{(n+1)} = \frac{n \cdot \vec{\mu}_k^{(n)} + \vec{d}_i}{n + 1}$$
$$\mathcal{E}_k^{(n+1)} = \mathcal{E}_k^{(n)} \cup \mathcal{E}_i$$
A new entry is appended to the event's internal chronological timeline.

---

## 4. Event Lifecycle & Evolution State Machine

Events are dynamic living phenomena that develop over days or weeks. GLOBALNEWS tracks each event through a deterministic Finite State Machine (FSM):

```
       +-----------------------+
       |       EMERGING        | <--- Initial state: single source, high velocity
       +-----------------------+
                   |
                   | Corroborated by >= 2 independent sources
                   v
       +-----------------------+
       |      DEVELOPING       | <--- Multi-source updates, expanding geographic reach
       +-----------------------+
              /         \
   Confirmed /           \ Conflicting / Refuted
            v             v
  +---------------+   +---------------+
  |   CONFIRMED   |   |   DISPUTED    | <--- Contested facts, official denials
  +---------------+   +---------------+
            \             /
   No update \           / Inactive > 72h
   for > 72h  v         v
       +-----------------------+
       |   HISTORICAL ARCHIVE  | <--- Fixed in SQLite, available for timeline replay
       +-----------------------+
```

### State Definitions
1. **`EMERGING`:** Detected within the last 60 minutes. Sourced from 1–2 outlets or an unverified social wire. Pulse frequency on the globe is rapid ($1.5\text{ Hz}$). Status banner: `EMERGING / UNVERIFIED`.
2. **`DEVELOPING`:** Corroborated by 3+ distinct publishers or an official institutional bulletin. Active new reports arriving every few minutes.
3. **`CONFIRMED`:** Validated by Tier 1 physical sensor data (USGS/GDACS) or official sovereign agencies. High evidence confidence ($C \ge 0.85$).
4. **`DISPUTED`:** Substantial contradictions detected across major outlets (e.g. conflicting attribution of missile strikes or disputed sovereignty claims). Marked with cadenced cadmium/white hatching on the globe.
5. **`STABILIZED / ARCHIVE`:** No new updates for $> 72\text{ hours}$. Visual pulse ceases; marker dims and becomes an archival historical node.

---

## 5. Event Significance & Hotspot Scoring Formula

A foundational principle of GLOBALNEWS is that **real significance is decoupled from media hype**.

### 5.1 The Master Hotspot Significance Formula
$$\text{Significance} = \min\left(10.0, \, \Big(w_{\text{dom}} \cdot I_{\text{domain}} + w_{\text{reach}} \cdot R_{\text{geo}} + w_{\text{sev}} \cdot S_{\text{severity}}\Big) \times C_{\text{evidence}} \times \Phi(N_{\text{sources}})\right)$$

Where:
* **$I_{\text{domain}} \in [1, 10]$:** Domain Base Impact:
  * Major Disasters / Armed Invasions: $9.0$
  * Epidemic Outbreaks / Nuclear Incidents: $8.5$
  * Major Scientific Breakthroughs (Peer-reviewed): $7.5$
  * Macroeconomic Shocks (Currency collapse, global rate hike): $7.0$
  * Routine Political Speeches / Bilateral Meetings: $3.0$
  * Sports / Celebrity Entertainment: $1.0$
* **$R_{\text{geo}} \in [1, 10]$:** Geographic & Demographic Reach (Local = 1, Regional = 3, National = 6, Continental = 8, Planetary = 10).
* **$S_{\text{severity}} \in [1, 10]$:** Concrete quantifiable severity (Casualties, economic dollar loss, infrastructure collapse).
* **$C_{\text{evidence}} \in [0.1, 1.0]$:** Evidence confidence score. Unverified rumors are scaled down to avoid false alarms.
* **$\Phi(N_{\text{sources}})$:** Non-linear logarithmic source volume dampener:
  $$\Phi(N) = 1.0 + 0.15 \cdot \log_2(1 + N_{\text{independent\_sources}})$$

### 5.2 Why the Logarithmic Dampener Prevents Media Distortion
* If $N = 1$ source: $\Phi(1) = 1.0 + 0.15 \cdot 1 = 1.15$
* If $N = 10$ sources: $\Phi(10) = 1.0 + 0.15 \cdot 3.45 = 1.51$
* If $N = 1,000$ syndicated spam copies: $\Phi(1000) = 1.0 + 0.15 \cdot 9.96 = 2.49$

Even a $1,000\times$ increase in article volume can never inflate a routine celebrity scandal ($I_{\text{domain}} = 1.0$) into competing with an un-hyped breakthrough in nuclear fusion ($I_{\text{domain}} = 7.5$) or an ongoing regional drought ($I_{\text{domain}} = 8.0$).

---

## 6. Event Relationship Graphs

Events do not exist in isolation. GLOBALNEWS models inter-event connectivity using typed edge relations:

```
[ Earthquake Mw 7.8 ] 
        |
        |--- (CAUSES / PRECIPITATES) ---> [ Tsunami Warning Honshu ]
        |
        |--- (DISRUPTS) ---------------> [ Tohoku Shinkansen Transit ]
        |
        |--- (PRECEDES) ---------------> [ JMA Disaster Cabinet Summit ]
```

### Relationship Types
1. **`CAUSES`:** Direct causal sequence (e.g. seismic rupture $\to$ tsunami).
2. **`DISRUPTS`:** Impact on secondary systems (e.g. port strike $\to$ shipping backlog).
3. **`ESCALATES`:** Intensification of existing tensions (e.g. border incident $\to$ troop deployment).
4. **`RESOLVES`:** De-escalation or mitigation (e.g. treaty signing $\to$ sanction relief).
5. **`FOLLOWS_UP`:** Ongoing administrative/legal aftermath.

