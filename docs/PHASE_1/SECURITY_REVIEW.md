# GLOBALNEWS — PHASE 1.2 SECURITY REVIEW
## Ingestion Pipeline Attack Surface, Threat Modeling & Defenses

**Document Version:** 1.0.0  
**Phase:** 1.2 Feed Ingestion and Normalization  
**Review Date:** November 9, 2026  
**Status:** AUDITED & VERIFIED  

---

### 1. Executive Summary

Ingesting untrusted remote content from external publisher feeds presents a substantial attack surface. In GLOBALNEWS Phase 1.2, every layer of retrieval, parsing, normalization, and local persistence was designed under zero-trust assumptions.

Four critical vulnerability vectors were identified, modeled, and mitigated with defense-in-depth:
1. **Server-Side Request Forgery (SSRF) & DNS Rebinding**
2. **XML External Entity (XXE) & Billion Laughs Expansion**
3. **Memory Exhaustion via Unbounded Payloads & Decompression Bombs**
4. **Content Injection & Database Corruption**

---

### 2. Threat Modeling & Implementation Defenses

#### 2.1 Server-Side Request Forgery (SSRF) & Loopback Traversal
* **Threat:** Malicious feed URLs pointing to `127.0.0.1`, `localhost`, AWS/GCP metadata endpoints (`169.254.169.254`), or private local area networks (`192.168.x.x`, `10.x.x.x`) to probe local daemons or exfiltrate host identity.
* **Defense Implementation (`SafeReqwestTransport`):**
  1. **Scheme Validation:** Only `http` and `https` schemes are accepted. Any attempt to use `file://`, `ftp://`, `gopher://`, or custom schemes is rejected with `IngestionError::InvalidUrl`.
  2. **IP Range Filtering:** The target hostname is resolved to socket IP addresses. Every resolved address is evaluated against standard RFC private and reserved address blocks:
     - `127.0.0.0/8` (IPv4 loopback)
     - `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16` (RFC 1918 private LANs)
     - `169.254.0.0/16` (Link-local / cloud metadata services)
     - `::1` (IPv6 loopback)
     - `fe80::/10` (IPv6 link-local)
     - `fc00::/7` (IPv6 unique local)
     - IPv4-mapped IPv6 addresses (`::ffff:127.0.0.1`)
  3. **Redirect Protection:** HTTP redirects are capped at 5 hops. Every redirect target URL is independently inspected through the SSRF filtering gate before request issuance.
  4. **DNS Rebinding Defense:** By resolving addresses prior to connection and pinning the target IP, time-of-check to time-of-use (TOCTOU) DNS rebinding is mitigated.

#### 2.2 XML External Entity (XXE) & Entity Expansion Attacks
* **Threat:** Feeding crafted XML payloads containing `<!ENTITY ... SYSTEM "file:///etc/passwd">` or recursive entity expansion (`<!ENTITY lol "lol"><!ENTITY lol2 "&lol;&lol;...">`) designed to cause local file disclosure or Denial of Service (DoS).
* **Defense Implementation (`Rss2Adapter`, `AtomAdapter`):**
  1. **Non-Resolving Stream Parser:** XML parsing is handled strictly via `quick-xml`, which operates in pure streaming mode without automated entity resolution or external DTD retrieval.
  2. **Proactive DOCTYPE / ENTITY Scanning:** Prior to parsing XML tree nodes, the raw payload is inspected for entity declarations:
     ```rust
     if payload.contains("<!ENTITY") || payload.contains("SYSTEM") {
         return Err(IngestionError::XmlParseError("Potential XXE entity declaration detected".into()));
     }
     ```
  3. **Unit Test Verification:** Verified deterministically via test `test_xxe_entity_expansion_defense`, which submits `tests/fixtures/xxe_attack.xml` and validates immediate rejection.

#### 2.3 Memory Exhaustion & Payload Size Caps
* **Threat:** Feed publishers or compromised CDNs streaming gigabyte-sized responses or nested gzip decompression bombs, exhausting daemon RAM.
* **Defense Implementation (`HttpTransport` & `SafeReqwestTransport`):**
  1. **Hard Size Limit:** An absolute threshold of `MAX_PAYLOAD_BYTES = 5 * 1024 * 1024` (5 MB) is enforced on all HTTP responses.
  2. **Chunked Stream Bounds:** `SafeReqwestTransport` streams response chunks incrementally into a pre-allocated buffer with a running counter. If the byte count exceeds 5,242,880 bytes, the stream is aborted immediately with `IngestionError::PayloadTooLarge`.
  3. **Memory Stability:** Prevents unbounded memory allocation before any DOM, JSON, or regex parsing begins.

#### 2.4 HTML Injection & Script Sanitization
* **Threat:** Malicious RSS feeds embedding executable JavaScript (`<script>`, `onerror=`, `iframe`) within `<title>` or `<description>` tags intended to exploit downstream WebViews or UI renderers.
* **Defense Implementation (`ArticleNormalizer`):**
  1. **Tag Stripping:** Regex and character entity decoders strip all HTML tags from titles and descriptions, converting raw HTML into clean plain-text excerpts.
  2. **Excerpt Truncation:** Descriptions are hard-truncated to a maximum of 500 characters, appending a clean ellipsis (`...`).
  3. **Structured Fields:** Coordinates, timestamps, and categories are parsed into strongly-typed primitives (`f64`, `DateTime<Utc>`, `SourceCategory` enum), preventing SQL injection or injection into frontend globe layers.

#### 2.5 Database Lock Contention & Async Thread Safety
* **Threat:** Holding exclusive SQLite write locks during network calls, causing lock contention, timeouts, and non-`Send` async futures.
* **Defense Implementation (`DatabaseManager` & `IngestionPipeline`):**
  1. **Pre-Fetch Separation:** Source health and feed checkpoints are read using concurrent, non-blocking `db.with_reader(...)` connections in SQLite WAL mode.
  2. **Zero Locks During IO:** Network retrieval (`transport.execute().await`) runs completely free of database locks.
  3. **Atomic Commit Scope:** The dedicated `db.get_writer()` write connection is acquired strictly after payload receipt and parsing, executing upserts and indexing within a brief single-threaded transaction.

---

### 3. Security Test Suite Summary

| Test Case | Attack Vector Tested | Expected Result | Status |
| :--- | :--- | :--- | :--- |
| `test_xxe_entity_expansion_defense` | Recursive XML entities (`<!ENTITY lol ...>`) | Immediate rejection (`XmlParseError`) | **PASS** |
| `test_payload_too_large_rejection` | Response payload > 5 MB | Early termination (`PayloadTooLarge`) | **PASS** |
| `test_http_429_backoff_handling` | Aggressive server polling | Exponential backoff window set in health table | **PASS** |
| `test_safe_transport_ssrf_blocking` | Loopback `127.0.0.1` and private IPs | SSRF connection rejection (`SsrfBlocked`) | **PASS** |
| `test_canonical_url_tracking_strip` | Marketing trackers & hash pollution | Clean canonical URLs & stable hashes | **PASS** |

