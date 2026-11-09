use globalnews_contracts::*;
use globalnews_core::ingestion::*;
use globalnews_storage::{DatabaseManager, Repository};
use std::sync::Arc;
use url::Url;

fn create_test_db() -> DatabaseManager {
    let file_name = format!("globalnews_pipeline_test_{}.db", uuid::Uuid::new_v4());
    let db_path = std::env::temp_dir().join(file_name);
    DatabaseManager::open_file(&db_path).unwrap()
}

// ---------------------------------------------------------------------------
// 1. Adapter & Parsing Tests
// ---------------------------------------------------------------------------

#[test]
fn test_rss2_parsing_and_normalization() {
    let xml = include_bytes!("fixtures/rss2_standard.xml");
    let adapter = Rss2Adapter;
    let items = adapter.parse(xml, "https://example.org/news").unwrap();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].title, "Volcanic Eruption Confirmed on Remote Pacific Island");
    assert!(items[0].link.contains("volcano-2026"));
    assert_eq!(items[0].authors, vec!["Jane Doe, Chief Geologist"]);
    assert_eq!(items[0].categories, vec!["Geology", "Natural Disaster"]);

    let source = Source::new(
        "test-disaster",
        "Test Disaster Publisher",
        1,
        "https://example.org/news/rss",
        "rss2",
        0.95,
    );

    let norm = ArticleNormalizer::normalize(&items[0], &source, None).unwrap();

    // Verify URL canonicalization stripped tracking query params & fragment
    assert_eq!(norm.canonical_url, "https://example.org/news/volcano-2026");
    assert_eq!(norm.original_url, "https://example.org/news/volcano-2026?utm_source=rss&utm_medium=feed#report");

    // Verify description excerpt has HTML tags cleanly removed
    assert_eq!(
        norm.description_excerpt.as_deref(),
        Some("Geologists reported a major ash plume rising to 12,000 meters. No casualties reported.")
    );

    // Verify timezone conversion: +0530 normalized to UTC
    let pub_utc = norm.published_at.unwrap();
    assert!(pub_utc.ends_with("+00:00") || pub_utc.ends_with('Z'));
    assert_eq!(norm.raw_published_at.as_deref(), Some("Mon, 09 Nov 2026 14:30:00 +0530"));
}

#[test]
fn test_atom_parsing_and_normalization() {
    let xml = include_bytes!("fixtures/atom_standard.xml");
    let adapter = AtomAdapter;
    let items = adapter.parse(xml, "https://example.org/humanitarian").unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Humanitarian Logistics Corridor Opened in Sahel Region");
    assert!(items[0].link.contains("sahel-corridor-2026"));
    assert_eq!(items[0].authors, vec!["UN OCHA Sahel Desk"]);

    let source = Source::new(
        "test-humanitarian",
        "Test Humanitarian Desk",
        2,
        "https://example.org/humanitarian/feed.atom",
        "atom",
        0.90,
    );

    let norm = ArticleNormalizer::normalize(&items[0], &source, None).unwrap();
    assert_eq!(norm.canonical_url, "https://example.org/humanitarian/reports/sahel-corridor-2026");
    assert!(norm.published_at.unwrap().contains("2026-11-09T11:45:00"));
}

#[test]
fn test_usgs_geojson_parsing_and_normalization() {
    let json = include_bytes!("fixtures/usgs_sample.json");
    let adapter = UsgsEarthquakeAdapter;
    let items = adapter.parse(json, "https://earthquake.usgs.gov").unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "M 6.8 - 124 km WSW of Honiara, Solomon Islands");
    assert_eq!(items[0].guid.as_deref(), Some("us7000sample1"));
    assert_eq!(items[0].extra_metadata.get("magnitude").map(|s| s.as_str()), Some("6.8"));

    let source = Source::new(
        "usgs-earthquakes",
        "US Geological Survey",
        1,
        "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_month.geojson",
        "geojson",
        1.0,
    );

    let norm = ArticleNormalizer::normalize(&items[0], &source, None).unwrap();
    assert_eq!(norm.source_id, "usgs-earthquakes");
    assert!(norm.description_excerpt.as_ref().unwrap().contains("Magnitude 6.8"));
}

#[test]
fn test_invalid_xml_error_handling() {
    let xml = include_bytes!("fixtures/invalid_xml.xml");
    let adapter = Rss2Adapter;
    let result = adapter.parse(xml, "https://example.org");
    assert!(result.is_err());
    match result.unwrap_err() {
        IngestionError::XmlParse(_) => {}
        other => panic!("Expected XmlParse error, got {:?}", other),
    }
}

#[test]
fn test_xxe_attack_defense() {
    let xml = include_bytes!("fixtures/xxe_attack.xml");
    let adapter = Rss2Adapter;
    let result = adapter.parse(xml, "https://example.org");
    assert!(result.is_err());
    match result.unwrap_err() {
        IngestionError::XmlParse(msg) => {
            assert!(msg.contains("XXE"));
        }
        other => panic!("Expected XXE XmlParse error, got {:?}", other),
    }
}

#[test]
fn test_missing_dates_honest_handling() {
    let xml = include_bytes!("fixtures/invalid_date.xml");
    let adapter = Rss2Adapter;
    let items = adapter.parse(xml, "https://example.org").unwrap();
    assert_eq!(items.len(), 1);

    let source = Source::new("test-src", "Test", 3, "https://example.org/feed", "rss2", 0.8);
    let norm = ArticleNormalizer::normalize(&items[0], &source, None).unwrap();

    // Published at is honestly None, raw string preserved
    assert_eq!(norm.published_at, None);
    assert_eq!(norm.raw_published_at.as_deref(), Some("Not-A-Valid-Timestamp-9999"));
}

// ---------------------------------------------------------------------------
// 2. HTTP Caching, Concurrency, and Resilience Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_http_304_not_modified_handling() {
    let db = Arc::new(create_test_db());
    let transport = MockHttpTransport::new();

    let source = Source::new("src-etag", "ETag Source", 2, "https://example.org/feed.xml", "rss2", 0.9);
    let endpoint = FeedEndpoint::new("ep-etag", "src-etag", "https://example.org/feed.xml", FeedFormat::Rss2);

    {
        let conn = db.get_writer().unwrap();
        Repository::upsert_source(&conn, &source).unwrap();
        Repository::upsert_feed_endpoint(&conn, &endpoint).unwrap();
        Repository::upsert_feed_checkpoint(
            &conn,
            &FeedCheckpoint {
                endpoint_id: endpoint.id.clone(),
                etag: Some("\"etag-v1\"".into()),
                last_modified: None,
                last_retrieval_at: Some("2026-11-09T10:00:00Z".into()),
                last_status_code: Some(200),
                last_successful_article_count: 5,
                content_hash: None,
            },
        ).unwrap();
    }

    // Register mock response with matching ETag
    transport.register(
        "https://example.org/feed.xml",
        HttpResponse {
            status_code: 200,
            etag: Some("\"etag-v1\"".into()),
            last_modified: None,
            retry_after_secs: None,
            content_type: Some("application/rss+xml".into()),
            body: include_bytes!("fixtures/rss2_standard.xml").to_vec(),
        },
    );

    let pipeline = IngestionPipeline::new(db.clone());
    let run = pipeline.ingest_endpoint(&transport, &source, &endpoint).await.unwrap();

    // Should return NotModified and 0 new articles
    assert_eq!(run.status, IngestionStatus::NotModified);
    assert_eq!(run.articles_fetched, 0);
    assert_eq!(run.articles_new, 0);
    assert_eq!(run.http_status_code, Some(304));
}

#[tokio::test]
async fn test_http_429_retry_after_backoff() {
    let db = Arc::new(create_test_db());
    let transport = MockHttpTransport::new();

    let source = Source::new("src-429", "Rate Limited Source", 3, "https://example.org/limited.xml", "rss2", 0.8);
    let endpoint = FeedEndpoint::new("ep-429", "src-429", "https://example.org/limited.xml", FeedFormat::Rss2);

    {
        let conn = db.get_writer().unwrap();
        Repository::upsert_source(&conn, &source).unwrap();
        Repository::upsert_feed_endpoint(&conn, &endpoint).unwrap();
    }

    transport.register(
        "https://example.org/limited.xml",
        HttpResponse {
            status_code: 429,
            etag: None,
            last_modified: None,
            retry_after_secs: Some(120),
            content_type: None,
            body: Vec::new(),
        },
    );

    let pipeline = IngestionPipeline::new(db.clone());
    let result = pipeline.ingest_endpoint(&transport, &source, &endpoint).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        IngestionError::RateLimited { retry_after_secs } => {
            assert_eq!(retry_after_secs, Some(120));
        }
        other => panic!("Expected RateLimited error, got {:?}", other),
    }

    // Verify SourceHealth recorded backoff
    let conn = db.get_reader().unwrap();
    let health = Repository::get_source_health(&conn, &source.id).unwrap().unwrap();
    assert_eq!(health.rate_limited_count, 1);
    assert_eq!(health.consecutive_failures, 1);
    assert!(health.backoff_until.is_some());
}

// ---------------------------------------------------------------------------
// 3. Deduplication and Multi-Publisher Independence Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_article_deduplication_exact_url() {
    let db = Arc::new(create_test_db());
    let transport = MockHttpTransport::new();

    let source = Source::new("src-dedup", "Dedup Source", 2, "https://example.org/dedup.xml", "rss2", 0.9);
    let endpoint = FeedEndpoint::new("ep-dedup", "src-dedup", "https://example.org/dedup.xml", FeedFormat::Rss2);

    {
        let conn = db.get_writer().unwrap();
        Repository::upsert_source(&conn, &source).unwrap();
        Repository::upsert_feed_endpoint(&conn, &endpoint).unwrap();
    }

    transport.register(
        "https://example.org/dedup.xml",
        HttpResponse {
            status_code: 200,
            etag: None,
            last_modified: None,
            retry_after_secs: None,
            content_type: Some("application/rss+xml".into()),
            body: include_bytes!("fixtures/rss2_standard.xml").to_vec(),
        },
    );

    let pipeline = IngestionPipeline::new(db.clone());

    // Run 1: 2 articles inserted
    let run1 = pipeline.ingest_endpoint(&transport, &source, &endpoint).await.unwrap();
    assert_eq!(run1.articles_fetched, 2);
    assert_eq!(run1.articles_new, 2);
    assert_eq!(run1.articles_duplicate, 0);

    // Run 2: Exact same feed items processed again
    let run2 = pipeline.ingest_endpoint(&transport, &source, &endpoint).await.unwrap();
    assert_eq!(run2.articles_fetched, 2);
    assert_eq!(run2.articles_new, 0);
    assert_eq!(run2.articles_duplicate, 2);
}

#[test]
fn test_independent_publishers_same_event() {
    let db = create_test_db();
    let mut conn = db.get_writer().unwrap();

    let src_a = Source::new("publisher-reuters", "Reuters", 2, "https://reuters.com/feed", "rss2", 0.95);
    let src_b = Source::new("publisher-ap", "Associated Press", 2, "https://apnews.com/feed", "rss2", 0.95);

    Repository::upsert_source(&conn, &src_a).unwrap();
    Repository::upsert_source(&conn, &src_b).unwrap();

    let raw_item_a = RawFeedItem {
        guid: Some("reuters-101".into()),
        title: "Peace Talks Commence in Geneva".into(),
        link: "https://reuters.com/article/geneva-talks-2026".into(),
        description: Some("Delegations arrived in Geneva today.".into()),
        pub_date_raw: Some("2026-11-09T10:00:00Z".into()),
        authors: vec!["Alice".into()],
        categories: vec!["Diplomacy".into()],
        ..Default::default()
    };

    let raw_item_b = RawFeedItem {
        guid: Some("ap-202".into()),
        title: "Peace Talks Commence in Geneva".into(),
        link: "https://apnews.com/article/geneva-talks-start".into(),
        description: Some("Envoys began negotiations in Switzerland.".into()),
        pub_date_raw: Some("2026-11-09T10:15:00Z".into()),
        authors: vec!["Bob".into()],
        categories: vec!["World".into()],
        ..Default::default()
    };

    let norm_a = ArticleNormalizer::normalize(&raw_item_a, &src_a, None).unwrap();
    let norm_b = ArticleNormalizer::normalize(&raw_item_b, &src_b, None).unwrap();

    let res_a = Repository::insert_normalized_article_if_unique(&mut conn, &norm_a).unwrap();
    let res_b = Repository::insert_normalized_article_if_unique(&mut conn, &norm_b).unwrap();

    // Both independent articles MUST be inserted!
    assert!(matches!(res_a, InsertResult::Inserted(_)));
    assert!(matches!(res_b, InsertResult::Inserted(_)));

    let fetched_a = Repository::get_normalized_article(&conn, &norm_a.id).unwrap().unwrap();
    let fetched_b = Repository::get_normalized_article(&conn, &norm_b.id).unwrap().unwrap();

    assert_eq!(fetched_a.source_id, "publisher-reuters");
    assert_eq!(fetched_b.source_id, "publisher-ap");
}

// ---------------------------------------------------------------------------
// 4. Security & Isolation Tests
// ---------------------------------------------------------------------------

#[test]
fn test_ssrf_defense_loopback_and_private() {
    assert!(SafeReqwestTransport::is_private_or_loopback_url(&Url::parse("http://127.0.0.1:8080/feed").unwrap()));
    assert!(SafeReqwestTransport::is_private_or_loopback_url(&Url::parse("http://localhost:3000/rss").unwrap()));
    assert!(SafeReqwestTransport::is_private_or_loopback_url(&Url::parse("http://10.0.0.1/admin").unwrap()));
    assert!(SafeReqwestTransport::is_private_or_loopback_url(&Url::parse("http://192.168.1.1/feed.xml").unwrap()));
    assert!(SafeReqwestTransport::is_private_or_loopback_url(&Url::parse("http://172.16.0.5/atom").unwrap()));
    assert!(SafeReqwestTransport::is_private_or_loopback_url(&Url::parse("http://[::1]/feed").unwrap()));

    assert!(!SafeReqwestTransport::is_private_or_loopback_url(&Url::parse("https://earthquake.usgs.gov/feed").unwrap()));
    assert!(!SafeReqwestTransport::is_private_or_loopback_url(&Url::parse("https://www.gdacs.org/xml/rss.xml").unwrap()));
}

#[tokio::test]
async fn test_error_isolation_between_sources() {
    let db = Arc::new(create_test_db());
    let transport = MockHttpTransport::new();

    let src1 = Source::new("src-broken", "Broken Source", 3, "https://example.org/broken.xml", "rss2", 0.5);
    let ep1 = FeedEndpoint::new("ep-broken", "src-broken", "https://example.org/broken.xml", FeedFormat::Rss2);

    let src2 = Source::new("src-good", "Good Source", 2, "https://example.org/good.xml", "rss2", 0.95);
    let ep2 = FeedEndpoint::new("ep-good", "src-good", "https://example.org/good.xml", FeedFormat::Rss2);

    {
        let conn = db.get_writer().unwrap();
        Repository::upsert_source(&conn, &src1).unwrap();
        Repository::upsert_feed_endpoint(&conn, &ep1).unwrap();
        Repository::upsert_source(&conn, &src2).unwrap();
        Repository::upsert_feed_endpoint(&conn, &ep2).unwrap();
    }

    // Register broken payload for EP 1 and valid for EP 2
    transport.register(
        "https://example.org/broken.xml",
        HttpResponse {
            status_code: 200,
            etag: None,
            last_modified: None,
            retry_after_secs: None,
            content_type: Some("application/rss+xml".into()),
            body: include_bytes!("fixtures/invalid_xml.xml").to_vec(),
        },
    );

    transport.register(
        "https://example.org/good.xml",
        HttpResponse {
            status_code: 200,
            etag: None,
            last_modified: None,
            retry_after_secs: None,
            content_type: Some("application/rss+xml".into()),
            body: include_bytes!("fixtures/rss2_standard.xml").to_vec(),
        },
    );

    let pipeline = IngestionPipeline::new(db.clone());

    // EP 1 fails
    let res1 = pipeline.ingest_endpoint(&transport, &src1, &ep1).await;
    assert!(res1.is_err());

    // EP 2 MUST succeed completely despite EP 1 failure!
    let res2 = pipeline.ingest_endpoint(&transport, &src2, &ep2).await;
    assert!(res2.is_ok());
    let run2 = res2.unwrap();
    assert_eq!(run2.status, IngestionStatus::Success);
    assert_eq!(run2.articles_new, 2);
}

#[test]
fn test_fts5_search_on_normalized_articles() {
    let db = create_test_db();
    let mut conn = db.get_writer().unwrap();

    let src = Source::new("src-search", "Search Pub", 1, "https://example.org/feed", "rss2", 1.0);
    Repository::upsert_source(&conn, &src).unwrap();

    let art1 = NormalizedArticle::new(
        "art-1",
        "src-search",
        "https://example.org/news/volcano",
        "https://example.org/news/volcano?ref=rss",
        "Catastrophic Volcanic Ash Plume Closes Airspace",
    );
    let mut art1_with_desc = art1;
    art1_with_desc.description_excerpt = Some("Seismologists recorded sustained tremors around the caldera.".into());
    art1_with_desc.source_provenance = "Ingested from Search Pub".into();

    let art2 = NormalizedArticle::new(
        "art-2",
        "src-search",
        "https://example.org/news/quantum",
        "https://example.org/news/quantum",
        "Major Quantum Teleportation Milestone in Optical Grid",
    );
    let mut art2_with_desc = art2;
    art2_with_desc.description_excerpt = Some("Physicists achieved 99.9% fidelity over 100 kilometers.".into());
    art2_with_desc.source_provenance = "Ingested from Search Pub".into();

    Repository::insert_normalized_article_if_unique(&mut conn, &art1_with_desc).unwrap();
    Repository::insert_normalized_article_if_unique(&mut conn, &art2_with_desc).unwrap();

    // Query 1: "Volcanic"
    let matches_volcano = Repository::search_normalized_articles_fts(&conn, "Volcanic", 10).unwrap();
    assert_eq!(matches_volcano.len(), 1);
    assert_eq!(matches_volcano[0].0, "art-1");

    // Query 2: "Quantum"
    let matches_quantum = Repository::search_normalized_articles_fts(&conn, "Quantum", 10).unwrap();
    assert_eq!(matches_quantum.len(), 1);
    assert_eq!(matches_quantum[0].0, "art-2");

    // Query 3: "caldera" (matched in description_excerpt!)
    let matches_caldera = Repository::search_normalized_articles_fts(&conn, "caldera", 10).unwrap();
    assert_eq!(matches_caldera.len(), 1);
    assert_eq!(matches_caldera[0].0, "art-1");
}
