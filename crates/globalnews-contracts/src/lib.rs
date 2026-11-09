pub mod article;
pub mod error;
pub mod events;
pub mod evidence;
pub mod ingestion;
pub mod location;
pub mod scoring;
pub mod source;
pub mod sync;

pub use article::{Article, ArticleFingerprint, ContentStorageClassification, NormalizedArticle};
pub use error::ContractError;
pub use events::{
    EventRelationship, EventStatus, MacroEvent, MacroEventStatus, MicroEvent, RelationType,
};
pub use evidence::{EvidenceRecord, TimelineEntry};
pub use ingestion::{ErrorKind, IngestionRun, IngestionStatus, InsertResult, RetrievalError};
pub use location::GeographicLocation;
pub use scoring::{ConfidenceScore, ImportanceScore};
pub use source::{
    FeedCheckpoint, FeedEndpoint, FeedFormat, RetrievalMethod, Source, SourceCategory, SourceHealth,
};
pub use sync::{SyncEnvelope, CURRENT_SCHEMA_VERSION};
