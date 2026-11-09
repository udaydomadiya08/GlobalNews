# GLOBALNEWS — Phase 1.1 Data Contract Implementation Specification

**Document ID:** `GN-DOC-P11-005`  
**Phase:** Phase 1.1 (Portable Rust Core Foundation & Cross-Platform Scaffolding)  
**Status:** Approved Technical Implementation Specification  
**Date:** 2026-11-09  

---

## 1. Data Contract Implementation Overview

The `globalnews-contracts` crate provides the canonical, strongly typed data layer across all platforms. Every struct implements:
1. `validate(&self) -> Result<(), ContractError>`: Enforces physical, mathematical, and semantic invariants.
2. `Serialize` and `Deserialize` (via `serde`): Guarantees lossless JSON/BSON roundtrips.
3. RFC 3339 UTC normalization for all temporal values.

---

## 2. Invariants & Validation Rules per Contract

```
+-------------------------------------------------------------------------------------------------------------------------+
|                                    DATA CONTRACT INVARIANTS TABLE                                                       |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| Contract Name        | Key Validated Fields        | Boundary Conditions               | Failure Action                 |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| **GeographicLocation**| `latitude`, `longitude`    | Lat: [-90.0, 90.0], Lon: [-180,180]| `InvalidLatitude / Longitude`  |
|                      | `country_code`              | Exactly 2 uppercase ASCII letters | `InvalidCountryCode`           |
|                      | `uncertainty_radius_km`     | Must be >= 0.0                    | `ValidationError`              |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| **ImportanceScore**  | `final_score`               | Must be in range [1.0, 10.0]      | `InvalidScoreRange`            |
|                      | `volume_dampener`           | Must be >= 1.0                    | `ValidationError`              |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| **ConfidenceScore**  | `final_confidence`          | Must be in range [0.1, 1.0]       | `InvalidScoreRange`            |
|                      | `jurisdictional_diversity`  | Must be in range [0.0, 1.0]       | `InvalidScoreRange`            |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| **Article**          | `url`                       | Must start with "http://" or "https"| `ValidationError`            |
|                      | `published_at`, `ingested`  | Valid RFC 3339 timestamp string   | `InvalidTimestamp`             |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| **Source**           | `tier`                      | Must be integer between 1 and 5   | `ValidationError`              |
|                      | `reliability_score`         | Must be in range [0.1, 1.0]       | `InvalidScoreRange`            |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| **MicroEvent**       | `id`, `title`               | Non-empty strings                 | `ValidationError`              |
|                      | `importance`, `confidence`  | Cascades to nested validators     | Bubbles up `ContractError`     |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| **MacroEvent**       | `id`, `title`               | Non-empty strings                 | `ValidationError`              |
|                      | `created_at`, `updated_at`  | Valid RFC 3339 timestamp strings  | `InvalidTimestamp`             |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| **EventRelationship**| `source_id`, `target_id`    | Cannot relate an event to itself  | `ValidationError`              |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
| **SyncEnvelope**     | `schema_version`            | Must strictly equal "1.0.0"       | `ValidationError`              |
+----------------------+-----------------------------+-----------------------------------+--------------------------------+
```

---

## 3. Serialization & Backward Compatibility Guarantees

* **Field Nullability:** Optional fields (`altitude_m`, `geonames_id`, `admin1_code`, `etag`, `last_modified`) use `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]` to produce clean, compact JSON payloads.
* **Enum Naming:** All status enums (`EventStatus`, `MacroEventStatus`, `RelationType`) serialize using `SCREAMING_SNAKE_CASE` (e.g. `"CONFIRMED"`, `"DEVELOPING"`, `"CAUSES"`), maintaining compatibility with Phase 0 API specifications.
* **Schema Evolution:** When contracts evolve, new optional fields may be introduced with default values (`#[serde(default)]`). Breaking schema changes require incrementing `CURRENT_SCHEMA_VERSION` (e.g. `"2.0.0"`).
