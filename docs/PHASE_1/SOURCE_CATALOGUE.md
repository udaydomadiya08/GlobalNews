# GLOBALNEWS — PHASE 1.2 SOURCE CATALOGUE
## Production Ingestion Registry & Verified Feed Endpoints

**Document Version:** 1.0.0  
**Phase:** 1.2 Ingestion & Normalization  
**Verification Date:** November 9, 2026  
**Status:** PRODUCTION VERIFIED  

---

### 1. Registry Architecture & Schema

In accordance with Phase 1.2 design requirements, GLOBALNEWS models sources as two distinct relational entities to support multiple feed formats and regional partitions per publisher:

1. **`sources` Table**: Publisher entity defining canonical domain, default category, tier, reliability score, and legal/licensing metadata.
2. **`feed_endpoints` Table**: Physical endpoint URLs belonging to a publisher, with distinct polling intervals, feed formats, HTTP headers, and activation states.

```
┌─────────────────────────────────┐       1 : N       ┌──────────────────────────────────────┐
│             sources             │ ────────────────> │            feed_endpoints            │
├─────────────────────────────────┤                   ├──────────────────────────────────────┤
│ id (PK)                         │                   │ id (PK)                              │
│ name                            │                   │ source_id (FK)                       │
│ canonical_domain                │                   │ endpoint_url                         │
│ category                        │                   │ feed_format (RSS2/Atom/JSON)         │
│ reliability_score               │                   │ polling_interval_secs                │
│ supported_languages             │                   │ enabled                              │
│ attribution_requirements        │                   │ created_at, updated_at               │
└─────────────────────────────────┘                   └──────────────────────────────────────┘
```

---

### 2. Verified Active Ingestion Endpoints

Every endpoint listed below has been verified via real HTTP probe and parser execution. No synthetic or unverified endpoints are permitted in Tier 1.

#### Endpoint 1: USGS Earthquakes Significant Monthly Feed (GeoJSON)
* **Publisher:** United States Geological Survey (USGS)
* **Source ID:** `src-usgs`
* **Canonical Domain:** `earthquake.usgs.gov`
* **Category:** `DisasterSeismic`
* **Tier:** 1 (Authoritative Primary Source)
* **Endpoint URL:** `https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_month.geojson`
* **Feed Format:** `JsonStructured` (GeoJSON FeatureCollection)
* **Default Polling Interval:** 600 seconds (10 minutes)
* **Retrieval Protocol:** HTTP GET with conditional headers (`If-Modified-Since`, `If-None-Match`)
* **Verified Response Status:** HTTP 200 OK
* **Supported Conditional Headers:** `Last-Modified` verified active
* **Licensing & Rights:** Public Domain / U.S. Government Work (CC0 equivalent). Open redistribution with attribution permitted.
* **Content Storage Classification:** `FullTextStored` (Public domain scientific event metadata).

#### Endpoint 2: USGS Earthquakes Significant Monthly Feed (Atom 1.0)
* **Publisher:** United States Geological Survey (USGS)
* **Source ID:** `src-usgs`
* **Canonical Domain:** `earthquake.usgs.gov`
* **Category:** `DisasterSeismic`
* **Tier:** 1 (Authoritative Primary Source)
* **Endpoint URL:** `https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_month.atom`
* **Feed Format:** `Atom` (XML Atom 1.0)
* **Default Polling Interval:** 900 seconds (15 minutes)
* **Verified Response Status:** HTTP 200 OK (`application/atom+xml`)
* **Licensing & Rights:** Public Domain / U.S. Government Work.
* **Content Storage Classification:** `FullTextStored`.

#### Endpoint 3: GDACS — Global Disaster Alert and Coordination System
* **Publisher:** Joint Research Centre (European Commission) & UN OCHA
* **Source ID:** `src-gdacs`
* **Canonical Domain:** `gdacs.org`
* **Category:** `DisasterGeneral`
* **Tier:** 1 (Multi-hazard Global Early Warning)
* **Endpoint URL:** `https://www.gdacs.org/xml/rss.xml`
* **Feed Format:** `Rss2` (RSS 2.0 with GeoRSS extensions)
* **Default Polling Interval:** 600 seconds (10 minutes)
* **Verified Response Status:** HTTP 200 OK; HTTP 304 Not Modified verified when passing `If-None-Match: "e4714d9b5770dd1:0"`.
* **Supported Conditional Headers:** Weak ETag (`W/"..."`), `Last-Modified`.
* **Licensing & Rights:** Open Access under EC / UN OCHA guidelines. Permitted for non-commercial and public alerting with GDACS citation.
* **Content Storage Classification:** `FullTextStored`.

#### Endpoint 4: ReliefWeb Humanitarian Updates
* **Publisher:** United Nations Office for the Coordination of Humanitarian Affairs (UN OCHA)
* **Source ID:** `src-reliefweb`
* **Canonical Domain:** `reliefweb.int`
* **Category:** `HumanitarianCrisis`
* **Tier:** 1 (Authoritative Global Humanitarian Reporting)
* **Endpoint URL:** `https://reliefweb.int/updates/rss.xml`
* **Feed Format:** `Rss2` (RSS 2.0)
* **Default Polling Interval:** 1800 seconds (30 minutes)
* **Verified Response Status:** HTTP 200 OK (`application/rss+xml`)
* **Licensing & Rights:** Creative Commons Attribution 4.0 International (CC BY 4.0). Excerpt and summary indexing permitted with link back.
* **Content Storage Classification:** `MetadataAndExcerptOnly`.

#### Endpoint 5: World Health Organization (WHO) News
* **Publisher:** World Health Organization
* **Source ID:** `src-who`
* **Canonical Domain:** `who.int`
* **Category:** `HealthEpidemic`
* **Tier:** 1 (Global Health Authority)
* **Endpoint URL:** `https://www.who.int/rss-feeds/news-english.xml`
* **Feed Format:** `Rss2` (RSS 2.0)
* **Default Polling Interval:** 3600 seconds (60 minutes)
* **Verified Response Status:** HTTP 200 OK (`application/rss+xml`)
* **Licensing & Rights:** WHO Open Access / WHO Copyright. Excerpt storage permitted with mandatory attribution and direct article link.
* **Content Storage Classification:** `MetadataAndExcerptOnly`.

#### Endpoint 6: BBC News World Edition
* **Publisher:** British Broadcasting Corporation (BBC)
* **Source ID:** `src-bbc`
* **Canonical Domain:** `bbc.co.uk`
* **Category:** `GeneralNews`
* **Tier:** 2 (Reputable International Broadcaster)
* **Endpoint URL:** `https://feeds.bbci.co.uk/news/world/rss.xml`
* **Feed Format:** `Rss2` (RSS 2.0)
* **Default Polling Interval:** 900 seconds (15 minutes)
* **Verified Response Status:** HTTP 200 OK (`text/xml; charset=utf-8`)
* **Licensing & Rights:** BBC Terms of Use. Non-commercial personal use only. Mandatory title + excerpt + canonical URL preservation. No raw full text storage.
* **Content Storage Classification:** `MetadataAndExcerptOnly` (Excerpt strictly capped at 500 characters).

---

### 3. Audited and Migrated Endpoints

| Publisher | Attempted Endpoint | Result | Root Cause & Resolution |
| :--- | :--- | :--- | :--- |
| **UN News** | `https://news.un.org/feed/subscribe/en/news/all/rss.xml` | **HTTP 404 Not Found** | UN News deprecated legacy RSS paths during web portal redesign. Replaced by verified UN OCHA ReliefWeb and WHO endpoints for Tier 1 international humanitarian coverage. |
| **Al Jazeera English** | `https://www.aljazeera.com/xml/rss/all.xml` | Under Evaluation | Cloudflare Bot Management challenges standard headless user agents. Under evaluation for Phase 1.3 adapter expansion. |

---

### 4. Attribution & Rights Classification Matrix

| Content Storage Classification | Eligible Sources | Stored Fields | Redistribution Policy |
| :--- | :--- | :--- | :--- |
| `FullTextStored` | USGS, NASA, Government Open Data | Title, Description, Body, GeoJSON, Raw Source | Publicly distributable with attribution |
| `MetadataAndExcerptOnly` | BBC, ReliefWeb, WHO, Reuters, AP | Title, Canonical URL, Published Date, Excerpt (≤500 chars) | Excerpt only; user click navigates to canonical web source |
| `TitleAndUrlOnly` | Restrictive paywalled / syndication sources | Title, Canonical URL, Source ID, Timestamp | Direct link only; zero excerpt |

