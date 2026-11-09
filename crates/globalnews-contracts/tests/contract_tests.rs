use globalnews_contracts::*;

#[test]
fn test_geographic_location_validation() {
    let loc = GeographicLocation::new(35.6762, 139.6503, "Tokyo", "JP", 5.0).unwrap();
    assert_eq!(loc.toponym_name, "Tokyo");
    assert_eq!(loc.country_code, "JP");

    // Invalid latitude
    assert!(GeographicLocation::new(95.0, 139.6503, "Tokyo", "JP", 5.0).is_err());
    // Invalid longitude
    assert!(GeographicLocation::new(35.6762, 195.0, "Tokyo", "JP", 5.0).is_err());
    // Invalid country code (lowercase)
    assert!(GeographicLocation::new(35.6762, 139.6503, "Tokyo", "jp", 5.0).is_err());
    // Negative uncertainty
    assert!(GeographicLocation::new(35.6762, 139.6503, "Tokyo", "JP", -1.0).is_err());
}

#[test]
fn test_importance_and_confidence_scoring() {
    let importance = ImportanceScore::new(8.0, 7.0, 6.0, 15).unwrap();
    assert!(importance.final_score >= 1.0 && importance.final_score <= 10.0);
    assert!(importance.volume_dampener > 1.0);

    let confidence = ConfidenceScore::new(5, 0.8, true, false).unwrap();
    assert!(confidence.final_confidence >= 0.85);
    assert!(confidence.has_sensor_validation);

    let disputed_conf = ConfidenceScore::new(5, 0.8, false, true).unwrap();
    assert!(disputed_conf.final_confidence < confidence.final_confidence);
}

#[test]
fn test_micro_event_serialization_roundtrip() {
    let loc = GeographicLocation::new(38.298, 142.373, "Miyagi Coast", "JP", 10.0).unwrap();
    let importance = ImportanceScore::new(9.0, 8.0, 7.5, 8).unwrap();
    let confidence = ConfidenceScore::new(4, 0.9, true, false).unwrap();

    let event = MicroEvent {
        id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        macro_event_id: Some("123e4567-e89b-12d3-a456-426614174000".to_string()),
        title: "Off-Coast Seismic Event Mw 7.2".to_string(),
        summary: "A magnitude 7.2 earthquake occurred off Honshu coast. Tsunami advisory issued."
            .to_string(),
        category_code: "03000000".to_string(),
        importance,
        confidence,
        location: loc,
        status: EventStatus::Confirmed,
        first_seen_at: "2026-11-09T06:00:00Z".to_string(),
        last_updated_at: "2026-11-09T06:30:00Z".to_string(),
        citation_count: 5,
    };

    assert!(event.validate().is_ok());

    let json = serde_json::to_string_pretty(&event).unwrap();
    assert!(json.contains("CONFIRMED"));
    assert!(json.contains("Miyagi Coast"));

    let deserialized: MicroEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}

#[test]
fn test_sync_envelope_validation() {
    let envelope = SyncEnvelope::new("node-alpha-123", None, vec![], vec![], vec![]);
    assert_eq!(envelope.schema_version, CURRENT_SCHEMA_VERSION);
    assert!(envelope.validate().is_ok());
}
