# GLOBALNEWS — Phase 0.2 Versioned Platform Data Contracts

**Document ID:** `GN-DOC-P02-004`  
**Phase:** Phase 0.2 (Cross-Platform Architecture Reconciliation)  
**Status:** Canonical Data Specification  
**Contract Version:** `1.0.0`  
**Date:** 2026-11-09  

---

## 1. Data Contract Philosophy & Guarantees

To ensure that the Desktop (Windows, macOS, Linux), Android, and iOS implementations interpret intelligence identically without semantic drift:
1. **Canonical Schema Definition:** All data structures are defined as versioned schemas with explicit nullability, unit definitions, and validation ranges.
2. **Deterministic Serialization:** Payloads exchanged across IPC, stored in SQLite, or exported for synchronization use strict JSON or BSON representations with RFC 3339 timestamps (ISO 8601 UTC).
3. **Rust `serde` Compatibility:** Every contract maps directly to a strongly typed Rust struct annotated with `#[derive(Serialize, Deserialize, Clone, Debug)]`.

---

## 2. Core Entity Data Contracts

### 2.1 GeographicLocation Contract
Represents a resolved physical coordinate with explicit uncertainty modeling.

```json
{
  "$schema": "https://globalnews.local/schemas/v1/geographic_location.json",
  "title": "GeographicLocation",
  "type": "object",
  "required": ["latitude", "longitude", "toponym_name", "country_code", "uncertainty_radius_km"],
  "properties": {
    "latitude": { "type": "number", "minimum": -90.0, "maximum": 90.0 },
    "longitude": { "type": "number", "minimum": -180.0, "maximum": 180.0 },
    "altitude_m": { "type": ["number", "null"] },
    "toponym_name": { "type": "string" },
    "geonames_id": { "type": ["integer", "null"] },
    "country_code": { "type": "string", "pattern": "^[A-Z]{2}$" },
    "admin1_code": { "type": ["string", "null"] },
    "uncertainty_radius_km": { "type": "number", "minimum": 0.0, "default": 10.0 }
  }
}
```

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GeographicLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: Option<f64>,
    pub toponym_name: String,
    pub geonames_id: Option<i64>,
    pub country_code: String, // ISO 3166-1 alpha-2
    pub admin1_code: Option<String>,
    pub uncertainty_radius_km: f64,
}
```

---

### 2.2 ImportanceScore Contract
Calculates the real-world significance decoupled from article volume.

```json
{
  "$schema": "https://globalnews.local/schemas/v1/importance_score.json",
  "title": "ImportanceScore",
  "type": "object",
  "required": ["final_score", "domain_severity", "geographic_reach", "physical_impact", "volume_dampener"],
  "properties": {
    "final_score": { "type": "number", "minimum": 1.0, "maximum": 10.0 },
    "domain_severity": { "type": "number", "minimum": 1.0, "maximum": 10.0 },
    "geographic_reach": { "type": "number", "minimum": 1.0, "maximum": 10.0 },
    "physical_impact": { "type": "number", "minimum": 0.0, "maximum": 10.0 },
    "volume_dampener": { "type": "number", "minimum": 1.0, "maximum": 3.0 }
  }
}
```

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImportanceScore {
    pub final_score: f64,        // Range: 1.0 to 10.0
    pub domain_severity: f64,    // IPTC base weight
    pub geographic_reach: f64,   // Local (1) to Planetary (10)
    pub physical_impact: f64,    // Casualties / Magnitude / Economic damage
    pub volume_dampener: f64,    // 1.0 + 0.15 * log2(1 + N_sources)
}
```

---

### 2.3 ConfidenceScore Contract
Quantifies evidence strength, source diversity, and jurisdictional corroborate.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ConfidenceScore {
    pub final_confidence: f64,       // Range: 0.1 to 1.0
    pub independent_source_count: u32,
    pub jurisdictional_diversity: f64, // 0.0 to 1.0 (cross-border corroboration)
    pub has_sensor_validation: bool,   // Verified via USGS/GDACS/NASA
    pub is_disputed: bool,             // Flagged if contradictory reports exist
}
```

---

### 2.4 Article Contract
Represents an individual document ingested from a publisher or sensor feed.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Article {
    pub id: String,                     // UUID v4
    pub micro_event_id: Option<String>, // Associated micro-event (nullable)
    pub url: String,                    // Canonical publisher URL
    pub title: String,                  // Cleaned title
    pub body_excerpt: String,           // 2-3 sentence factual TextRank extract
    pub language: String,               // ISO 639-1 code (e.g. "en", "fr")
    pub source_id: String,              // Foreign key to Source
    pub content_hash: String,           // SHA-256 hash of normalized text
    pub published_at: String,           // RFC 3339 UTC
    pub ingested_at: String,            // RFC 3339 UTC
    pub is_syndicated: bool,            // True if identified as wire duplicate
}
```

---

### 2.5 Source Contract
Represents an authorized publisher, governmental agency, or sensor network.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Source {
    pub id: String,                     // E.g. "usgs-earthquakes", "reuters-world"
    pub name: String,                   // Display name
    pub tier: u8,                       // Tier 1 (Sensor) to Tier 5 (Science)
    pub feed_url: String,               // Endpoint URL
    pub feed_type: String,              // "geojson", "rss2", "atom", "reliefweb_api"
    pub reliability_score: f64,         // Base reputation weight (0.1 to 1.0)
    pub etag: Option<String>,           // HTTP caching header
    pub last_modified: Option<String>,  // HTTP caching header
    pub last_polled_at: Option<String>, // RFC 3339 UTC
}
```

---

### 2.6 MicroEvent Contract (Incidents)
Represents a localized real-world development plotted as a hotspot on the 3D Earth.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MicroEvent {
    pub id: String,                     // UUID v4
    pub macro_event_id: Option<String>, // Parent container (e.g. war or crisis)
    pub title: String,                  // Canonical event headline
    pub summary: String,                // 3-sentence synthesized extractive dossier
    pub category_code: String,          // Standard IPTC Media Topic (e.g. "03000000")
    pub importance: ImportanceScore,    // Drives hotspot radius
    pub confidence: ConfidenceScore,    // Drives hotspot opacity / cadence
    pub location: GeographicLocation,   // Spherical plotting coordinates
    pub status: EventStatus,            // Emerging, Developing, Confirmed, Disputed, Archive
    pub first_seen_at: String,          // RFC 3339 UTC
    pub last_updated_at: String,        // RFC 3339 UTC
    pub citation_count: u32,            // Number of distinct citations
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventStatus {
    Emerging,
    Developing,
    Confirmed,
    Disputed,
    Archive,
}
```

---

### 2.7 MacroEvent Contract (Megastories)
Represents persistent, multi-month or multi-year global themes.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MacroEvent {
    pub id: String,                     // UUID v4
    pub title: String,                  // E.g. "Russo-Ukrainian War"
    pub category_code: String,          // IPTC code
    pub summary: String,                // Ongoing macro overview
    pub status: MacroEventStatus,       // Active, Dormant, Concluded
    pub bounding_box: [f64; 4],         // [min_lat, min_lon, max_lat, max_lon]
    pub created_at: String,             // RFC 3339 UTC
    pub updated_at: String,             // RFC 3339 UTC
    pub active_micro_event_count: u32,  // Number of child incidents
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MacroEventStatus {
    Active,
    Dormant,
    Concluded,
}
```

---

### 2.8 EventRelationship Contract
Models relational directed edges connecting distinct events.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EventRelationship {
    pub source_event_id: String,
    pub target_event_id: String,
    pub relationship_type: RelationType,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationType {
    Causes,     // Earthquake Mw 7.2 -> Tsunami Warning
    Disrupts,   // Port Strike -> Supply Chain Backlog
    Escalates,  // Border Clashes -> Diplomatic Recall
    Resolves,   // Ceasefire Treaty -> Sanctions Relief
    Precedes,   // Bilateral Summit -> Trade Agreement
}
```

---

### 2.9 TimelineEntry & EvidenceRecord Contracts

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TimelineEntry {
    pub id: String,
    pub micro_event_id: String,
    pub timestamp: String,              // RFC 3339 UTC
    pub title: String,
    pub description: String,
    pub citation_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EvidenceRecord {
    pub id: String,
    pub micro_event_id: String,
    pub source_id: String,
    pub article_id: String,
    pub corroboration_weight: f64,
}
```

---

### 2.10 SyncEnvelope Contract (Peer & Client Synchronization)

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SyncEnvelope {
    pub schema_version: String,         // "1.0.0"
    pub client_id: String,              // Ephemeral installation UUID
    pub generated_at: String,           // RFC 3339 UTC
    pub last_sync_cursor: Option<String>,// High-water mark timestamp
    pub micro_events: Vec<MicroEvent>,
    pub macro_events: Vec<MacroEvent>,
    pub relationships: Vec<EventRelationship>,
}
```

