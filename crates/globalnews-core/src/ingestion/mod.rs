pub mod adapter;
pub mod error;
pub mod normalizer;
pub mod pipeline;
pub mod resilience;
pub mod transport;

pub use adapter::{AtomAdapter, FeedAdapter, RawFeedItem, Rss2Adapter, UsgsEarthquakeAdapter};
pub use error::IngestionError;
pub use normalizer::ArticleNormalizer;
pub use pipeline::IngestionPipeline;
pub use resilience::{BackoffPolicy, ConcurrencyLimiter};
pub use transport::{
    HttpRequest, HttpResponse, HttpTransport, MockHttpTransport, SafeReqwestTransport,
    MAX_RESPONSE_BYTES,
};
