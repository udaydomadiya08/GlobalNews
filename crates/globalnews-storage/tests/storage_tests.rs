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
