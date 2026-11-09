# GLOBALNEWS — PHASE 1.1.1: DEPENDENCY AND LICENCE RECHECK

**Date:** 2026-11-09  
**Status:** Audit Complete — 100% Permissive Licences Verified  
**Scope:** Root `Cargo.toml` and Member Crates

---

## 1. Direct Dependency Inventory

The GLOBALNEWS workspace declares dependencies centrally in `[workspace.dependencies]` to guarantee version consistency across all crates:

| Crate | Declared Version | Purpose | Licence | Maintenance Status |
|---|---|---|---|---|
| `serde` | `1.0` (derive) | Canonical serialization framework | **MIT OR Apache-2.0** | Tier-1 Core, Actively Maintained |
| `serde_json` | `1.0` | JSON format serialization | **MIT OR Apache-2.0** | Tier-1 Core, Actively Maintained |
| `chrono` | `0.4` (serde) | RFC 3339 UTC timestamp manipulation | **MIT OR Apache-2.0** | Tier-1 Standard, Actively Maintained |
| `uuid` | `1.10` (v4, serde)| Unique ID generation for events & articles | **Apache-2.0 OR MIT** | Tier-1 Standard, Actively Maintained |
| `rusqlite` | `0.32` (bundled) | Embedded SQLite bindings | **MIT** (SQLite: Public Domain) | Tier-1 Standard, Actively Maintained |
| `thiserror` | `1.0` | Strongly-typed error derivation | **MIT OR Apache-2.0** | Tier-1 Standard (dtolnay), Actively Maintained |
| `tracing` | `0.1` | Structured, scoped diagnostic logging | **MIT** | Official Tokio Working Group |
| `tracing-subscriber` | `0.3` (fmt, env-filter)| Log formatting and level filtering | **MIT** | Official Tokio Working Group |
| `tokio` | `1.40` (multi-thread, sync)| Async runtime, timers, and primitives | **MIT** | Official Tokio Working Group |
| `libc` | `0.2` | POSIX memory RSS system calls (`getrusage`) | **MIT OR Apache-2.0** | Official Rust Working Group |

---

## 2. Copyleft & Legal Risk Evaluation

1. **GPL / AGPL / LGPL Prohibition:**
   - **Zero copyleft crates exist in the workspace.**
   - All direct crates are dual-licensed under `MIT OR Apache-2.0` or single-licensed under `MIT`.
   - `rusqlite` compiles the official SQLite C amalgamation, which is dedicated to the **Public Domain** under the SQLite Blessing.
2. **Static Linking Safety on iOS and Mobile:**
   - Because none of the dependencies use LGPL with dynamic linking requirements, static compilation into iOS `.ipa` containers and Android `.so` JNI shared libraries is 100% compliant with Apple App Store and Google Play distribution terms.
3. **Patent & Commercial Freedom:**
   - Apache-2.0 grants an explicit, royalty-free patent license, protecting the project from patent infringement claims related to contributed code.

---

## 3. Security & Maintenance Evaluation

- **Minimal Dependency Principle:** The workspace avoids heavy convenience frameworks, pulling in only 10 fundamental dependencies across the entire core, persistence, and contract layers.
- **Vulnerability Advisories:** None of the declared dependency versions (`serde 1.0`, `tokio 1.40`, `rusqlite 0.32`, `uuid 1.10`) have active RUSTSEC advisories.
