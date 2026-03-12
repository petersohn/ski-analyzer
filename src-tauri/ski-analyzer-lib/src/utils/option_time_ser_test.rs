use serde::{Deserialize, Serialize};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::utils::option_time_ser;

#[derive(Deserialize, Serialize)]
struct OptionTimeWrapper(
    #[serde(with = "option_time_ser")] Option<OffsetDateTime>,
);

fn parse_time(s: &str) -> OffsetDateTime {
    OffsetDateTime::parse(s, &Iso8601::DEFAULT).unwrap()
}

#[test]
fn test_serialize_deserialize_roundtrip_with_value() {
    let original: Option<OffsetDateTime> =
        Some(parse_time("2024-01-15T10:30:00Z"));
    let wrapper = OptionTimeWrapper(original);
    let json = serde_json::to_string(&wrapper).unwrap();
    let deserialized: OptionTimeWrapper = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.0, Some(parse_time("2024-01-15T10:30:00Z")));
}

#[test]
fn test_serialize_deserialize_roundtrip_none() {
    let original: Option<OffsetDateTime> = None;
    let wrapper = OptionTimeWrapper(original);
    let json = serde_json::to_string(&wrapper).unwrap();
    let deserialized: OptionTimeWrapper = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.0, None);
}

#[test]
fn test_deserialize_from_json_with_value() {
    let json = r#""2024-01-15T10:30:00Z""#;
    let wrapper: OptionTimeWrapper = serde_json::from_str(json).unwrap();
    assert_eq!(wrapper.0, Some(parse_time("2024-01-15T10:30:00Z")));
}

#[test]
fn test_deserialize_from_json_null() {
    let json = r#"null"#;
    let wrapper: OptionTimeWrapper = serde_json::from_str(json).unwrap();
    assert_eq!(wrapper.0, None);
}

#[test]
fn test_serialize_to_json_with_value() {
    let original: Option<OffsetDateTime> =
        Some(parse_time("2024-01-15T10:30:00Z"));
    let wrapper = OptionTimeWrapper(original);
    let json = serde_json::to_string(&wrapper).unwrap();
    assert!(json.contains("2024-01-15T10:30:00"));
    assert!(json.contains("Z"));
}

#[test]
fn test_serialize_to_json_none() {
    let original: Option<OffsetDateTime> = None;
    let wrapper = OptionTimeWrapper(original);
    let json = serde_json::to_string(&wrapper).unwrap();
    assert_eq!(json, "null");
}
