use globalnews_contracts::*;
use globalnews_core::ingestion::*;
use globalnews_storage::{DatabaseManager, Repository};
use std::sync::Arc;

#[tokio::test]
#[ignore = "Live external smoke test; requires live outbound internet and is excluded from deterministic CI"]
async fn test_live_usgs_earthquake_ingestion_smoke() {
    let file_name = format!("globalnews_live_smoke_{}.db", uuid::Uuid::new_v4());
    let db_path = std::env::temp_dir().join(file_name);
    let db = Arc::new(DatabaseManager::open_file(&db_path).unwrap());

    let source = Source::new(
        "usgs-earthquakes-live",
        "USGS Earthquake Hazards Program (Live Smoke)",
        1,
        "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_month.geojson",
        "geojson",
        1.0,
    );

    let endpoint = FeedEndpoint::new(
        "ep-usgs-live",
        "usgs-earthquakes-live",
        "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_month.geojson",
        FeedFormat::JsonStructured,
    );

    {
        let conn = db.get_writer().unwrap();
        Repository::upsert_source(&conn, &source).unwrap();
        Repository::upsert_feed_endpoint(&conn, &endpoint).unwrap();
    }

    let transport = SafeReqwestTransport::new().expect("Failed to build SafeReqwestTransport");
    let pipeline = IngestionPipeline::new(db.clone());

    // 1. First Live Ingestion Run
    let run1 = pipeline
        .ingest_endpoint(&transport, &source, &endpoint)
        .await
        .expect("Live USGS ingestion failed");

    assert_eq!(run1.status, IngestionStatus::Success);
    assert_eq!(run1.http_status_code, Some(200));
    println!(
        "Live Ingestion Run 1: fetched={}, new={}, duplicate={}",
        run1.articles_fetched, run1.articles_new, run1.articles_duplicate
    );

    // 2. Second Live Ingestion Run (Should deduplicate or 304)
    let run2 = pipeline
        .ingest_endpoint(&transport, &source, &endpoint)
        .await
        .expect("Second live ingestion run failed");

    println!(
        "Live Ingestion Run 2: status={:?}, fetched={}, new={}, duplicate={}",
        run2.status, run2.articles_fetched, run2.articles_new, run2.articles_duplicate
    );

    if run2.status == IngestionStatus::Success {
        // If server sent full payload, all articles must be deduplicated
        assert_eq!(run2.articles_new, 0);
        assert_eq!(run2.articles_duplicate, run1.articles_fetched);
    } else {
        // Or server responded with 304 Not Modified
        assert_eq!(run2.status, IngestionStatus::NotModified);
    }

    // Cleanup temporary files
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
}

