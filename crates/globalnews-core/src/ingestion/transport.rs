use super::error::IngestionError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use url::Url;

pub const MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024; // 5 MB ceiling

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub timeout: Duration,
}

impl HttpRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            etag: None,
            last_modified: None,
            timeout: Duration::from_secs(15),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HttpResponse {
    pub status_code: u16,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub retry_after_secs: Option<u64>,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, IngestionError>;
}

// ===========================================================================
// 1. Mock Transport for Deterministic Offline Tests
// ===========================================================================

pub struct MockHttpTransport {
    responses: Arc<Mutex<HashMap<String, HttpResponse>>>,
    call_log: Arc<Mutex<Vec<String>>>,
}

impl MockHttpTransport {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register(&self, url: impl Into<String>, response: HttpResponse) {
        let mut map = self.responses.lock().unwrap();
        map.insert(url.into(), response);
    }

    pub fn call_count(&self, url_prefix: &str) -> usize {
        let log = self.call_log.lock().unwrap();
        log.iter().filter(|u| u.starts_with(url_prefix)).count()
    }
}

impl Default for MockHttpTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpTransport for MockHttpTransport {
    async fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, IngestionError> {
        {
            let mut log = self.call_log.lock().unwrap();
            log.push(req.url.clone());
        }

        let map = self.responses.lock().unwrap();
        if let Some(resp) = map.get(&req.url) {
            // Check conditional matching if registered as 304 capable
            if let Some(ref req_etag) = req.etag {
                if let Some(ref resp_etag) = resp.etag {
                    if req_etag == resp_etag {
                        return Ok(HttpResponse {
                            status_code: 304,
                            etag: resp.etag.clone(),
                            last_modified: resp.last_modified.clone(),
                            retry_after_secs: None,
                            content_type: None,
                            body: Vec::new(),
                        });
                    }
                }
            }

            if resp.status_code == 429 {
                return Err(IngestionError::RateLimited {
                    retry_after_secs: resp.retry_after_secs,
                });
            }

            if resp.status_code >= 400 {
                return Err(IngestionError::HttpStatus {
                    status: resp.status_code,
                    message: format!("HTTP error {} on {}", resp.status_code, req.url),
                });
            }

            Ok(resp.clone())
        } else {
            Err(IngestionError::HttpStatus {
                status: 404,
                message: format!("Mock URL not registered: {}", req.url),
            })
        }
    }
}

// ===========================================================================
// 2. Safe Live Transport with SSRF Defense & Bounded Payloads
// ===========================================================================

pub struct SafeReqwestTransport {
    client: reqwest::Client,
}

impl SafeReqwestTransport {
    pub fn new() -> Result<Self, IngestionError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 5 {
                    attempt.error("Too many redirects (max 5)")
                } else {
                    let next_url = attempt.url();
                    if SafeReqwestTransport::is_private_or_loopback_url(next_url) {
                        attempt.error("SSRF redirect blocked to private network")
                    } else {
                        attempt.follow()
                    }
                }
            }))
            .build()
            .map_err(|e| IngestionError::Http(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self { client })
    }

    /// Evaluates whether an IP address belongs to RFC 1918, loopback, link-local, or documentation blocks.
    pub fn is_private_or_loopback_ip(ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => {
                v4.is_loopback() // 127.0.0.0/8
                    || v4.is_private() // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                    || v4.is_link_local() // 169.254.0.0/16
                    || v4.is_broadcast()
                    || v4.is_documentation()
                    || v4.octets()[0] == 0 // 0.0.0.0/8
            }
            IpAddr::V6(v6) => {
                v6.is_loopback() // ::1
                    || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 (Unique Local)
                    || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 (Link-local)
            }
        }
    }

    /// Checks URL destination against SSRF restrictions.
    pub fn is_private_or_loopback_url(url: &Url) -> bool {
        let host = match url.host_str() {
            Some(h) => h,
            None => return true,
        };

        if host.eq_ignore_ascii_case("localhost") || host.ends_with(".local") {
            return true;
        }

        // Direct IP parse
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Self::is_private_or_loopback_ip(ip);
        }

        // DNS resolution check
        let port = url.port_or_known_default().unwrap_or(80);
        let socket_str = format!("{}:{}", host, port);
        if let Ok(addrs) = socket_str.to_socket_addrs() {
            for addr in addrs {
                if Self::is_private_or_loopback_ip(addr.ip()) {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for SafeReqwestTransport {
    fn default() -> Self {
        Self::new().expect("Failed to initialize SafeReqwestTransport")
    }
}

#[async_trait]
impl HttpTransport for SafeReqwestTransport {
    async fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, IngestionError> {
        let parsed_url = Url::parse(&req.url)
            .map_err(|e| IngestionError::Http(format!("Malformed URL '{}': {}", req.url, e)))?;

        // 1. SSRF Defense
        if Self::is_private_or_loopback_url(&parsed_url) {
            return Err(IngestionError::SsrfBlocked(format!(
                "Host in URL '{}' resolves to prohibited private or loopback destination",
                req.url
            )));
        }

        // 2. Build Request
        let mut builder = self.client.get(&req.url);

        if let Some(ref etag) = req.etag {
            builder = builder.header(reqwest::header::IF_NONE_MATCH, etag);
        }
        if let Some(ref last_mod) = req.last_modified {
            builder = builder.header(reqwest::header::IF_MODIFIED_SINCE, last_mod);
        }

        builder = builder.header(
            reqwest::header::USER_AGENT,
            "GlobalNewsBot/0.1.0 (+https://github.com/udaydomadiya08/GlobalNews; research intelligence bot)",
        );

        let response = builder
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    IngestionError::Timeout(e.to_string())
                } else {
                    IngestionError::Http(e.to_string())
                }
            })?;

        let status = response.status().as_u16();

        // 3. Handle HTTP 304 Not Modified
        if status == 304 {
            let etag = response
                .headers()
                .get(reqwest::header::ETAG)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let last_mod = response
                .headers()
                .get(reqwest::header::LAST_MODIFIED)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            return Ok(HttpResponse {
                status_code: 304,
                etag,
                last_modified: last_mod,
                retry_after_secs: None,
                content_type: None,
                body: Vec::new(),
            });
        }

        // 4. Handle HTTP 429 Too Many Requests
        if status == 429 {
            let retry_after = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            return Err(IngestionError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        // 5. Handle HTTP Errors
        if status >= 400 {
            return Err(IngestionError::HttpStatus {
                status,
                message: format!("HTTP error {} on {}", status, req.url),
            });
        }

        // 6. Extract Headers
        let etag = response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let last_mod = response
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // 7. Bounded Body Read
        let bytes = response
            .bytes()
            .await
            .map_err(|e| IngestionError::Http(format!("Failed to read response body: {}", e)))?;

        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(IngestionError::PayloadTooLarge {
                size_bytes: bytes.len(),
                max_bytes: MAX_RESPONSE_BYTES,
            });
        }

        Ok(HttpResponse {
            status_code: status,
            etag,
            last_modified: last_mod,
            retry_after_secs: None,
            content_type,
            body: bytes.to_vec(),
        })
    }
}
