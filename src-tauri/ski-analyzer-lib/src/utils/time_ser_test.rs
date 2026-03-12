use serde::{Deserialize, Serialize};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::utils::time_ser;

#[derive(Deserialize, Serialize)]
struct TimeWrapper(#[serde(with = "time_ser")] OffsetDateTime);

fn parse_time(s: &str) -> OffsetDateTime {
    OffsetDateTime::parse(s, &Iso8601::DEFAULT).unwrap()
}

#[test]
fn test_serialize_deserialize_roundtrip() {
    let original = parse_time("2024-01-15T10:30:00Z");
    let wrapper = TimeWrapper(original);
    let json = serde_json::to_string(&wrapper).unwrap();
    let deserialized: TimeWrapper = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.0, original);
}

#[test]
fn test_deserialize_from_json() {
    let json = r#""2024-01-15T10:30:00Z""#;
    let wrapper: TimeWrapper = serde_json::from_str(json).unwrap();
    let expected = parse_time("2024-01-15T10:30:00Z");
    assert_eq!(wrapper.0, expected);
}

#[test]
fn test_serialize_to_json() {
    let original = parse_time("2024-01-15T10:30:00Z");
    let wrapper = TimeWrapper(original);
    let json = serde_json::to_string(&wrapper).unwrap();
    assert!(json.contains("2024-01-15T10:30:00"));
    assert!(json.contains("Z"));
}
