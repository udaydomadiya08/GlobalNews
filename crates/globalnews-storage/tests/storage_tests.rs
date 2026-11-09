use globalnews_contracts::*;
use globalnews_storage::*;

#[test]
fn test_database_initialization_and_migrations() {
    let db = DatabaseManager::new_in_memory().expect("Failed to initialize in-memory DB");
    db.with_writer(|conn| {
        let version = MigrationRunner::get_current_version(conn)?;
        assert_eq!(version, 1, "Expected migration version 1 to be applied");
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_macro_and_micro_event_crud() {
    let db = DatabaseManager::new_in_memory().unwrap();

    let macro_ev = MacroEvent {
        id: "macro-001".into(),
        title: "Global Supply Chain Reorganization".into(),
        category_code: "04000000".into(),
        summary: "Multi-national maritime trade realignments".into(),
        status: MacroEventStatus::Active,
        bounding_box: [-60.0, -180.0, 60.0, 180.0],
        created_at: "2026-11-09T00:00:00Z".into(),
        updated_at: "2026-11-09T06:00:00Z".into(),
        active_micro_event_count: 1,
    };

    db.with_writer(|conn| {
        Repository::insert_macro_event(conn, &macro_ev)?;
        let retrieved = Repository::get_macro_event(conn, "macro-001")?.expect("Macro event not found");
        assert_eq!(retrieved.title, "Global Supply Chain Reorganization");
        assert_eq!(retrieved.status, MacroEventStatus::Active);
        Ok(())
    })
    .unwrap();

    let loc = GeographicLocation::new(1.290270, 103.851959, "Port of Singapore", "SG", 5.0).unwrap();
    let importance = ImportanceScore::new(8.5, 9.0, 7.0, 12).unwrap();
    let confidence = ConfidenceScore::new(6, 0.95, false, false).unwrap();

    let micro_ev = MicroEvent {
        id: "micro-001".into(),
        macro_event_id: Some("macro-001".into()),
        title: "Malacca Strait Shipping Advisory".into(),
        summary: "Maritime authorities adjust navigation routes in response to naval exercises.".into(),
        category_code: "04000000".into(),
        importance,
        confidence,
        location: loc,
        status: EventStatus::Developing,
        first_seen_at: "2026-11-09T04:00:00Z".into(),
        last_updated_at: "2026-11-09T06:15:00Z".into(),
        citation_count: 6,
    };

    db.with_writer(|conn| {
        Repository::insert_micro_event(conn, &micro_ev)?;
        let retrieved = Repository::get_micro_event(conn, "micro-001")?.expect("Micro event not found");
        assert_eq!(retrieved.title, "Malacca Strait Shipping Advisory");
        assert_eq!(retrieved.status, EventStatus::Developing);
        assert_eq!(retrieved.macro_event_id, Some("macro-001".into()));
        assert_eq!(retrieved.location.toponym_name, "Port of Singapore");
        assert_eq!(retrieved.location.country_code, "SG");
        assert_eq!(retrieved.importance.final_score, importance.final_score);
        assert_eq!(retrieved.confidence.final_confidence, confidence.final_confidence);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_fts5_bm25_search() {
    let db = DatabaseManager::new_in_memory().unwrap();

    let loc1 = GeographicLocation::new(35.68, 139.76, "Tokyo", "JP", 10.0).unwrap();
    let ev1 = MicroEvent {
        id: "ev-tokyo".into(),
        macro_event_id: None,
        title: "Magnitude 6.5 Offshore Earthquake Strikes Honshu".into(),
        summary: "Seismic sensors detect severe shaking; high-speed rail safely paused.".into(),
        category_code: "03000000".into(),
        importance: ImportanceScore::new(8.0, 7.0, 6.0, 5).unwrap(),
        confidence: ConfidenceScore::new(4, 0.8, true, false).unwrap(),
        location: loc1,
        status: EventStatus::Confirmed,
        first_seen_at: "2026-11-09T05:00:00Z".into(),
        last_updated_at: "2026-11-09T05:10:00Z".into(),
        citation_count: 3,
    };

    let loc2 = GeographicLocation::new(46.20, 6.14, "Geneva", "CH", 5.0).unwrap();
    let ev2 = MicroEvent {
        id: "ev-geneva".into(),
        macro_event_id: None,
        title: "Diplomatic Treaty on Climate Mitigation Concluded in Geneva".into(),
        summary: "Delegates finalize carbon reduction frameworks following multi-day plenary session.".into(),
        category_code: "06000000".into(),
        importance: ImportanceScore::new(7.0, 9.0, 5.0, 20).unwrap(),
        confidence: ConfidenceScore::new(8, 0.99, false, false).unwrap(),
        location: loc2,
        status: EventStatus::Confirmed,
        first_seen_at: "2026-11-09T04:30:00Z".into(),
        last_updated_at: "2026-11-09T06:00:00Z".into(),
        citation_count: 15,
    };

    db.with_writer(|conn| {
        Repository::insert_micro_event(conn, &ev1)?;
        Repository::insert_micro_event(conn, &ev2)?;

        // Search for "Earthquake"
        let results = Repository::search_events(conn, "Earthquake", 10)?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "ev-tokyo");

        // Search for "Diplomatic Treaty"
        let results = Repository::search_events(conn, "Diplomatic Treaty", 10)?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "ev-geneva");

        Ok(())
    })
    .unwrap();
}

#[test]
fn test_persistent_database_lifecycle_and_wal() {
    let db_path = std::env::temp_dir().join(format!("globalnews_test_wal_{}.db", uuid::Uuid::new_v4()));
    
    // 1. First session: initialize and verify WAL mode
    {
        let db = DatabaseManager::open_file(&db_path).expect("Failed to open persistent database");
        db.with_writer(|conn| {
            let journal_mode: String = conn.query_row("PRAGMA journal_mode;", [], |r| r.get(0))?;
            assert_eq!(journal_mode.to_lowercase(), "wal", "Expected WAL journal mode");

            let busy_timeout: i64 = conn.query_row("PRAGMA busy_timeout;", [], |r| r.get(0))?;
            assert_eq!(busy_timeout, 5000, "Expected 5000ms busy timeout");

            let loc = GeographicLocation::new(40.7128, -74.0060, "New York", "US", 5.0).unwrap();
            let importance = ImportanceScore::new(8.0, 8.0, 7.0, 10).unwrap();
            let confidence = ConfidenceScore::new(5, 0.9, true, false).unwrap();

            let ev = MicroEvent {
                id: "ev-persistent-01".into(),
                macro_event_id: None,
                title: "UN General Assembly Special Session".into(),
                summary: "Delegates assemble to address global infrastructure security.".into(),
                category_code: "11000000".into(),
                importance,
                confidence,
                location: loc,
                status: EventStatus::Confirmed,
                first_seen_at: "2026-11-09T08:00:00Z".into(),
                last_updated_at: "2026-11-09T09:00:00Z".into(),
                citation_count: 8,
            };

            Repository::insert_micro_event(conn, &ev)?;
            Ok(())
        })
        .unwrap();
    } // DatabaseManager dropped here (connections closed)

    // 2. Second session: reopen database from disk and verify persistence
    {
        let db = DatabaseManager::open_file(&db_path).expect("Failed to reopen persistent database");
        db.with_reader(|conn| {
            let retrieved = Repository::get_micro_event(conn, "ev-persistent-01")?
                .expect("Persistent event not found after database reopen");
            assert_eq!(retrieved.title, "UN General Assembly Special Session");
            assert_eq!(retrieved.location.toponym_name, "New York");
            assert_eq!(retrieved.location.country_code, "US");
            assert_eq!(retrieved.citation_count, 8);

            // Verify FTS5 search persists after reopen
            let results = Repository::search_events(conn, "Assembly", 10)?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].0, "ev-persistent-01");
            Ok(())
        })
        .unwrap();
    }

    // Cleanup temporary files
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
}

#[test]
fn test_fts5_triggers_on_update_and_delete() {
    let db = DatabaseManager::new_in_memory().unwrap();

    let loc = GeographicLocation::new(51.5074, -0.1278, "London", "GB", 5.0).unwrap();
    let importance = ImportanceScore::new(7.0, 7.0, 5.0, 5).unwrap();
    let confidence = ConfidenceScore::new(3, 0.7, false, false).unwrap();

    let ev = MicroEvent {
        id: "ev-trigger-test".into(),
        macro_event_id: None,
        title: "Major River Thames Flash Flood Warning".into(),
        summary: "Rising water levels prompt emergency flood barrier deployment.".into(),
        category_code: "03000000".into(),
        importance,
        confidence,
        location: loc.clone(),
        status: EventStatus::Emerging,
        first_seen_at: "2026-11-09T08:00:00Z".into(),
        last_updated_at: "2026-11-09T08:30:00Z".into(),
        citation_count: 2,
    };

    db.with_writer(|conn| {
        // 1. Insert: FTS trigger micro_events_ai fires
        Repository::insert_micro_event(conn, &ev)?;

        let hits = Repository::search_events(conn, "Thames", 10)?;
        assert_eq!(hits.len(), 1, "Expected match for 'Thames'");

        let hits_flood = Repository::search_events(conn, "Flood", 10)?;
        assert_eq!(hits_flood.len(), 1, "Expected match for 'Flood'");

        // 2. Update: FTS trigger micro_events_au fires
        let mut updated_ev = ev.clone();
        updated_ev.title = "London Transport Network Normalization Advisory".into();
        updated_ev.summary = "Subway lines resume standard operating schedule following inspection.".into();
        updated_ev.last_updated_at = "2026-11-09T09:00:00Z".into();

        Repository::insert_micro_event(conn, &updated_ev)?;

        // Old keyword "Flood" must now produce 0 matches!
        let hits_old = Repository::search_events(conn, "Flood", 10)?;
        assert_eq!(hits_old.len(), 0, "Old keyword 'Flood' must be purged from FTS5 index upon update");

        // New keyword "Subway" must now produce 1 match!
        let hits_new = Repository::search_events(conn, "Subway", 10)?;
        assert_eq!(hits_new.len(), 1, "New keyword 'Subway' must match after FTS5 trigger update");

        // 3. Delete: FTS trigger micro_events_ad fires
        let deleted = Repository::delete_micro_event(conn, "ev-trigger-test")?;
        assert!(deleted, "Expected micro event to be deleted");

        // FTS search for "Subway" must now produce 0 matches!
        let hits_deleted = Repository::search_events(conn, "Subway", 10)?;
        assert_eq!(hits_deleted.len(), 0, "Deleted event must be purged from FTS5 index");

        Ok(())
    })
    .unwrap();
}

#[test]
fn test_migration_rollback_on_failure() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    
    // Ensure migrations table exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, name TEXT, applied_at TEXT);",
        [],
    ).unwrap();

    // Run an intentionally failing transaction
    let attempt = (|| -> Result<(), rusqlite::Error> {
        let tx = conn.transaction()?;
        tx.execute("CREATE TABLE test_rollback (id INTEGER PRIMARY KEY);", [])?;
        // Intentionally induced syntax error
        tx.execute("INVALID SQL STATEMENT SYNTAX ERROR", [])?;
        tx.commit()?;
        Ok(())
    })();

    assert!(attempt.is_err(), "Expected transaction to fail");

    // Verify test_rollback table does NOT exist because transaction rolled back cleanly
    let table_exists: bool = conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='test_rollback';",
        [],
        |_| Ok(true),
    ).unwrap_or(false);

    assert!(!table_exists, "Table must not exist after transaction rollback");
}

#[test]
fn test_macro_event_relationships() {
    let db = DatabaseManager::new_in_memory().unwrap();

    let loc = GeographicLocation::new(14.5995, 120.9842, "Manila", "PH", 10.0).unwrap();
    let importance = ImportanceScore::new(8.0, 7.0, 7.0, 10).unwrap();
    let confidence = ConfidenceScore::new(5, 0.9, true, false).unwrap();

    let ev1 = MicroEvent {
        id: "ev-typhoon".into(),
        macro_event_id: None,
        title: "Category 5 Super Typhoon Landfall".into(),
        summary: "Coastal communities evacuate ahead of extreme cyclonic winds.".into(),
        category_code: "03000000".into(),
        importance: importance.clone(),
        confidence: confidence.clone(),
        location: loc.clone(),
        status: EventStatus::Confirmed,
        first_seen_at: "2026-11-09T08:00:00Z".into(),
        last_updated_at: "2026-11-09T08:30:00Z".into(),
        citation_count: 10,
    };

    let ev2 = MicroEvent {
        id: "ev-power-grid".into(),
        macro_event_id: None,
        title: "Regional Electric Grid Islanding and Outage".into(),
        summary: "Transmission interconnects trip automatically to protect core infrastructure.".into(),
        category_code: "04000000".into(),
        importance,
        confidence,
        location: loc,
        status: EventStatus::Confirmed,
        first_seen_at: "2026-11-09T09:00:00Z".into(),
        last_updated_at: "2026-11-09T09:15:00Z".into(),
        citation_count: 4,
    };

    let relationship = EventRelationship {
        source_event_id: "ev-typhoon".into(),
        target_event_id: "ev-power-grid".into(),
        relationship_type: RelationType::Causes,
        description: Some("Typhoon storm surge and gale gusts caused transmission line failures.".into()),
    };

    db.with_writer(|conn| {
        Repository::insert_micro_event(conn, &ev1)?;
        Repository::insert_micro_event(conn, &ev2)?;
        Repository::insert_relationship(conn, &relationship)?;

        let rels = Repository::get_relationships_for_event(conn, "ev-typhoon")?;
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].source_event_id, "ev-typhoon");
        assert_eq!(rels[0].target_event_id, "ev-power-grid");
        assert_eq!(rels[0].relationship_type, RelationType::Causes);

        Ok(())
    })
    .unwrap();
}

#[test]
fn test_concurrent_readers_and_single_writer() {
    let db = DatabaseManager::new_in_memory().unwrap();
    let db_arc = std::sync::Arc::new(db);

    // Seed 1 record
    let loc = GeographicLocation::new(0.0, 0.0, "Origin", "XX", 1.0).unwrap();
    let ev = MicroEvent {
        id: "ev-seed".into(),
        macro_event_id: None,
        title: "Initial Seeding Event".into(),
        summary: "Seed event summary.".into(),
        category_code: "01000000".into(),
        importance: ImportanceScore::new(5.0, 5.0, 5.0, 1).unwrap(),
        confidence: ConfidenceScore::new(1, 0.5, false, false).unwrap(),
        location: loc,
        status: EventStatus::Confirmed,
        first_seen_at: "2026-11-09T00:00:00Z".into(),
        last_updated_at: "2026-11-09T00:00:00Z".into(),
        citation_count: 1,
    };

    db_arc.with_writer(|conn| {
        Repository::insert_micro_event(conn, &ev)?;
        Ok(())
    }).unwrap();

    let mut handles = Vec::new();

    // Spawn 4 concurrent reader threads
    for _ in 0..4 {
        let db_clone = std::sync::Arc::clone(&db_arc);
        let handle = std::thread::spawn(move || {
            for _ in 0..20 {
                db_clone.with_reader(|conn| {
                    let ev = Repository::get_micro_event(conn, "ev-seed")?;
                    assert!(ev.is_some());
                    Ok(())
                }).unwrap();
            }
        });
        handles.push(handle);
    }

    // Main thread performs concurrent writes
    for i in 0..10 {
        let mut loop_ev = ev.clone();
        loop_ev.id = format!("ev-concurrent-{}", i);
        loop_ev.title = format!("Concurrent Event {}", i);
        db_arc.with_writer(|conn| {
            Repository::insert_micro_event(conn, &loop_ev)?;
            Ok(())
        }).unwrap();
    }

    for h in handles {
        h.join().unwrap();
    }
}
