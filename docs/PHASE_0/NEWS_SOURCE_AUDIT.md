# GLOBALNEWS — Worldwide News & Intelligence Source Audit

**Document ID:** `GN-DOC-P0-003`  
**Phase:** Phase 0 (Research & Feasibility)  
**Status:** Approved Architectural Baseline  
**Inspection Date:** 2026-11-09  
**Coverage Target:** Global multi-lingual coverage across 6 continents, 15+ languages, institutional alerts, and scientific feeds.

---

## 1. Multi-Tiered Source Architecture

To guarantee high coverage while respecting our non-negotiable principles (no mandatory paid APIs, no single point of failure), GLOBALNEWS adopts a **5-tier ingestion taxonomy**:

```
+---------------------------------------------------------------------------------------------------+
| TIER 1: GROUND-TRUTH SENSOR & DISASTER FEEDS (Automated, Structured GeoJSON, Zero Bias)           |
| -> USGS (Earthquakes) | GDACS (Disasters) | NASA EONET (Natural Events) | NOAA (Severe Weather)   |
+---------------------------------------------------------------------------------------------------+
                                                  |
+---------------------------------------------------------------------------------------------------+
| TIER 2: INSTITUTIONAL & INTERGOVERNMENTAL FEEDS (Official, High-Authority Declarations)            |
| -> UN News | WHO Outbreaks | IAEA Nuclear | WMO Climate | World Bank / IMF Reports                |
+---------------------------------------------------------------------------------------------------+
                                                  |
+---------------------------------------------------------------------------------------------------+
| TIER 3: GLOBAL & REGIONAL WIRE SERVICES & PUBLISHERS (Direct RSS, Continuous Human Reporting)     |
| -> Reuters | BBC | AP | AFP | Al Jazeera | DW | France24 | NHK | ToI | AllAfrica | MercoPress    |
+---------------------------------------------------------------------------------------------------+
                                                  |
+---------------------------------------------------------------------------------------------------+
| TIER 4: GLOBAL KNOWLEDGE GRAPH & OPEN REPOSITORIES (15-Minute Macro-Index)                         |
| -> GDELT 2.0 (DOC API, GEO API, GKG Translingual File Stream)                                    |
+---------------------------------------------------------------------------------------------------+
                                                  |
+---------------------------------------------------------------------------------------------------+
| TIER 5: SCIENTIFIC DISCOVERY & ACADEMIC PREPRINTS (Peer-Reviewed, Breakthrough Intelligence)      |
| -> arXiv.org | Nature News RSS | ScienceDaily | PubMed Trending                                  |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Exhaustive Audit of Source Candidates

### 2.1 Tier 1: Ground-Truth Disaster & Environmental Feeds

#### A. Global Disaster Alert and Coordination System (GDACS)
* **Access Method:** REST API (`GET https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH`) & GeoRSS (`https://www.gdacs.org/xml/rss.xml`).
* **Geographic Coverage:** 100% Worldwide.
* **Language:** English (primary), multilingual standardized codes.
* **Update Frequency:** Every 6 minutes.
* **Payload Format:** Pure GeoJSON with standardized hazard types: `EQ` (Earthquakes), `TC` (Tropical Cyclones), `FL` (Floods), `VO` (Volcanoes), `DR` (Droughts), `WF` (Wildfires).
* **Alert Levels:** Native semantic alert levels: `Red`, `Orange`, `Green`.
* **Rate Limits:** Max 100 records per page; no authentication key required.
* **Licensing & Terms:** Free open data under UN/European Commission mandate. Attribution required: *"Global Disaster Alert and Coordination System (GDACS)"*.
* **Reliability:** 99.9% uptime; multi-datacenter institutional backing.
* **Gaps:** Excludes armed conflicts and political events.
* **Historical Data:** Supported via `fromdate`/`todate` query parameters spanning 20+ years.

#### B. USGS Earthquake Hazards Program
* **Access Method:** Real-time GeoJSON endpoints (`https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_day.geojson`).
* **Geographic Coverage:** Global seismic network.
* **Update Frequency:** Real-time (stream updated within 60 seconds of seismic event).
* **Payload Format:** Strict GeoJSON FeatureCollection with magnitude, depth, coordinates, tsunami flag, and alert tier (PAGER).
* **Rate Limits:** Public unauthenticated endpoint, polite caching recommended (poll interval: 60–120s).
* **Licensing:** US Government Public Domain (100% free and open for commercial/public use).
* **Historical Data:** Complete queryable archive since 1970 via FDSN web service.

#### C. NASA EONET v3 (Earth Observatory Natural Event Tracker)
* **Access Method:** REST API (`https://eonet.gsfc.nasa.gov/api/v3/events?status=open`).
* **Geographic Coverage:** Global satellite tracking.
* **Update Frequency:** Hourly updates.
* **Event Categories:** Wildfires, severe storms, volcanoes, sea/lake ice, floods.
* **Rate Limits:** Generous public access; no API key required.
* **Licensing:** NASA Open Data Policy (Public Domain).

---

### 2.2 Tier 2: Institutional & Intergovernmental Feeds

| Organization | Feed Endpoint | Update Cadence | Focus Area | Reliability |
| :--- | :--- | :--- | :--- | :--- |
| **United Nations (UN News)** | `https://news.un.org/feed/subscribe/en/news/all/rss.xml` | 15–30 mins | Geopolitics, humanitarian aid, treaties | Very High |
| **WHO Outbreak News** | `https://www.who.int/feeds/entity/csr/don/en/rss.xml` | Daily / Ad-hoc | Epidemics, pathogen alerts, biosafety | Authoritative |
| **IAEA (Atomic Energy)** | `https://www.iaea.org/feeds/top-stories` | Daily | Nuclear facilities, radiological safety | Authoritative |
| **WMO (Meteorological)** | `https://wmo.int/news/feed` | Daily | Global climate records, extreme weather | High |
| **World Bank News** | `https://www.worldbank.org/en/news/all/rss` | Daily | Economic indicators, developing nations | High |

---

### 2.3 Tier 3: Global Wire Services & Multi-Continent Publishers

To prevent Western or single-state geographic bias, direct RSS feeds are aggregated across 6 continents and multiple major world languages:

```
+------------------+------------------------+--------------------------------------------------------+
| Region           | Publisher / Outlet     | RSS Endpoint                                           |
+------------------+------------------------+--------------------------------------------------------+
| International    | Reuters World          | Direct Wire / Open Syndicated Feeds                    |
| International    | Associated Press (AP)  | AP World News Hub                                      |
| International    | Agence France-Presse   | AFP News RSS (FR, EN, ES, AR)                          |
| Europe           | BBC World              | http://feeds.bbci.co.uk/news/world/rss.xml             |
| Europe           | Deutsche Welle         | https://rss.dw.com/xml/rss-en-world                    |
| Europe           | France 24              | https://www.france24.com/en/rss                        |
| Middle East      | Al Jazeera English     | https://www.aljazeera.com/xml/rss/all.xml              |
| Middle East      | Al Arabiya (Arabic)    | https://www.alarabiya.net/.well-known/rss              |
| Asia-Pacific     | NHK World Japan        | https://www3.nhk.or.jp/rss/news/cat0.xml               |
| Asia-Pacific     | Times of India         | https://timesofindia.indiatimes.com/rssfeedstopstories |
| Asia-Pacific     | South China Morning P. | https://www.scmp.com/rss/91/feed                       |
| Latin America    | MercoPress             | https://en.mercopress.com/rss/                         |
| Latin America    | Telesur (Spanish)      | https://www.telesurtv.net/rss/                         |
| Africa           | AllAfrica Global       | https://allafrica.com/tools/headlines/rdf/latest.rdf   |
| North America    | CBC News               | https://www.cbc.ca/cxml/rss/rss-topstories.xml         |
+------------------+------------------------+--------------------------------------------------------+
```

* **Access Method:** HTTP/2 GET with standard conditional `ETag` and `If-Modified-Since` headers.
* **Rate Limits:** Polling cadence throttled to 15–30 minutes per feed to prevent server throttling and avoid abusive traffic.
* **Licensing & Legal Status:** Ingestion conforms to standard Fair Use doctrine: GLOBALNEWS stores and displays article titles, timestamps, 2-to-3 sentence extractive summaries, detected geographic coordinates, and canonical outbound links to the publisher. Full publisher articles are never redistributed.

---

### 2.4 Tier 4: GDELT 2.0 (Global Database of Events, Language, and Tone)

* **Access Method 1 (DOC 2.0 API):** `https://api.gdeltproject.org/api/v2/doc/doc?query=...&mode=artlist&format=json`
* **Access Method 2 (Raw 15-Minute Data Stream):** Every 15 minutes, GDELT uploads three compressed CSV files indexed at `http://data.gdeltproject.org/gdeltv2/lastupdate.txt`:
  1. `export.CSV.zip` (Cameo coded event records: conflict, diplomacy, protest).
  2. `mentions.CSV.zip` (Article citation links and sentiment).
  3. `gkg.CSV.zip` (Global Knowledge Graph: themes, locations, organizations, persons).
* **Coverage:** Worldwide; translates from 65+ languages into English.
* **Rate Limits (API):** Enforces rate limiting on consecutive queries (HTTP 429 triggered if requests are spaced closer than 5 seconds). The 15-minute file download stream has no rate-limiting issues when fetched once per cycle.
* **Licensing:** 100% Free and open data under GDELT terms; attribution required.
* **Gaps & Weaknesses:**
  * High false-positive rate on geographic toponyms (e.g. historical mentions of "Georgia" classified as the Caucasus nation rather than the US State).
  * Automated translation artifacts occasionally mischaracterize peaceful diplomatic events as hostilities.
* **Role in GLOBALNEWS:** **Used as a macro-validation and secondary corroboration layer, NOT as the sole source of truth.**

---

### 2.5 Tier 5: Scientific Breakthroughs & Academic Preprints

| Source | Endpoint | Update Frequency | Significance Threshold |
| :--- | :--- | :--- | :--- |
| **arXiv.org** | `http://export.arxiv.org/rss/` (cs, physics, bio, astro) | Daily (00:00 UTC) | Cross-citation velocity |
| **Nature News** | `https://www.nature.com/nature.rss` | Daily | Major peer-reviewed breakthroughs |
| **ScienceDaily** | `https://www.sciencedaily.com/rss/all.xml` | Continuous | General scientific progress |
| **PubMed Trending**| `https://pubmed.ncbi.nlm.nih.gov/rss/trending/` | Weekly | Medical and bioscience developments |

---

## 3. Critical Flaws in Naive Aggregation Strategies

### 3.1 Why Google News Unofficial RSS is Rejected as a Primary Source
Many hobbyist aggregators rely on undocumented Google News RSS URLs (e.g. `news.google.com/rss/search?q=...`).
* **Fatal Flaw 1 (Aggressive IP Rate-Limiting & Bot Blocking):** Google regularly blocks requests originating from residential or non-browser user agents with HTTP 429 or CAPTCHA challenges.
* **Fatal Flaw 2 (URL Obfuscation):** Google News wraps all source URLs in proprietary base64 redirection tokens (`news.google.com/rss/articles/CBMi...`) that change frequently and require automated headless browser redirection to resolve to canonical publisher domains, violating our lightweight, zero-overhead principles.
* **Decision:** **Do not use Google News as a core infrastructure feed.**

### 3.2 Why GDELT Alone Is Insufficient
GDELT provides immense global scale, but relies on fully automated machine processing without human editorial curation:
1. Significant time delay on small regional breaking news (up to 45–90 minutes).
2. CAMEO event coding often misclassifies routine administrative rhetoric as aggressive threats.
3. Lack of canonical original article text summaries.
* **Decision:** **GDELT must be cross-checked against Tier 1 sensor data and Tier 3 publisher RSS feeds.**

---

## 4. Multi-Source Redundancy & Cross-Corroboration Matrix

To calculate the **Evidence Confidence Score ($C_{\text{evidence}}$)** for any event hotspot, GLOBALNEWS evaluates source overlap across independent categories:

```
                  +-------------------------------------------------+
                  |             INCOMING EVENT CANDIDATE            |
                  +-------------------------------------------------+
                                           |
             +-----------------------------+-----------------------------+
             v                                                           v
+--------------------------+                               +--------------------------+
|  Sensor / Alert Source   |                               |  Independent Wire Feed   |
|  (e.g., USGS or GDACS)   |                               |  (e.g., Reuters, BBC)    |
|  Weight: +0.45           |                               |  Weight: +0.30 per wire  |
+--------------------------+                               +--------------------------+
             \                                                           /
              +----------------------------+----------------------------+
                                           v
                         +-----------------------------------+
                         | Cross-Corroboration Engine        |
                         | Total Confidence: C in [0.0, 1.0] |
                         +-----------------------------------+
```

### Confidence Formulation
$$C_{\text{evidence}} = \min\left(1.0, \sum_{i \in \text{Sources}} w_i \cdot \text{Reputation}_i \cdot \text{DiversityFactor}\right)$$

Where:
* Tier 1 Sensor Feeds: $w = 0.50$ (objective physical data).
* Tier 2 Institutional Feeds: $w = 0.40$.
* Tier 3 Major Verified Wires: $w = 0.25$ each.
* Tier 4 GDELT GKG Mentions: $w = 0.15$.
* $\text{DiversityFactor} = 1.2$ if sources span multiple geographic jurisdictions; $0.7$ if all sources originate from a single national broadcaster.

