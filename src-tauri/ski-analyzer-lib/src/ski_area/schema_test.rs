use super::{
    Difficulty, Lift, Piste, PisteData, PisteMetadata, PointWithElevation,
    SkiArea, SkiAreaMetadata,
};
use crate::utils::bounded_geometry::BoundedGeometry;
use geo::{LineString, MultiLineString, MultiPolygon, Point, Polygon, Rect};
use jsonschema::Validator;
use schemars::schema_for;
use std::collections::HashMap;

fn create_test_point_with_elevation() -> PointWithElevation {
    PointWithElevation::new(Point::new(1.0, 2.0), 1000)
}

fn create_test_lift() -> Lift {
    let line_points = vec![(0.0, 0.0), (1.0, 1.0)];
    Lift {
        ref_: "1".to_string(),
        name: "Test Lift".to_string(),
        type_: "chair_lift".to_string(),
        line: BoundedGeometry::new(LineString::from(line_points)).unwrap(),
        stations: vec![create_test_point_with_elevation()],
        can_go_reverse: false,
        can_disembark: false,
        lengths: vec![100.0],
    }
}

fn create_test_piste_metadata() -> PisteMetadata {
    PisteMetadata {
        ref_: "1".to_string(),
        name: "Test Piste".to_string(),
        difficulty: Difficulty::Intermediate,
    }
}

fn create_test_piste_data() -> PisteData {
    let exterior = LineString::from(vec![
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
        (0.0, 0.0),
    ]);
    let polygon = Polygon::new(exterior, vec![]);
    let multi_polygon = MultiPolygon::new(vec![polygon]);

    PisteData {
        bounding_rect: Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
        areas: multi_polygon,
        lines: MultiLineString::new(vec![]),
    }
}

fn create_test_piste() -> Piste {
    Piste {
        metadata: create_test_piste_metadata(),
        data: create_test_piste_data(),
    }
}

fn create_test_ski_area_metadata() -> SkiAreaMetadata {
    let exterior = LineString::from(vec![
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
        (0.0, 0.0),
    ]);
    let polygon = Polygon::new(exterior, vec![]);

    SkiAreaMetadata {
        id: 12345,
        name: "Test Ski Area".to_string(),
        outline: BoundedGeometry::new(polygon).unwrap(),
    }
}

fn create_test_ski_area() -> SkiArea {
    let mut lifts = HashMap::new();
    lifts.insert("1".to_string(), create_test_lift());

    let mut pistes = HashMap::new();
    pistes.insert("1".to_string(), create_test_piste());

    SkiArea {
        metadata: create_test_ski_area_metadata(),
        lifts,
        pistes,
        bounding_rect: Rect::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
        date: time::OffsetDateTime::from_unix_timestamp(1700000000).unwrap(),
    }
}

#[test]
fn validate_point_with_elevation_schema() {
    let schema = schema_for!(PointWithElevation);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let point_with_ele = create_test_point_with_elevation();
    let json = serde_json::to_value(&point_with_ele).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_lift_schema() {
    let schema = schema_for!(Lift);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let lift = create_test_lift();
    let json = serde_json::to_value(&lift).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_difficulty_schema() {
    let schema = schema_for!(Difficulty);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let difficulty = Difficulty::Intermediate;
    let json = serde_json::to_value(&difficulty).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_piste_metadata_schema() {
    let schema = schema_for!(PisteMetadata);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let metadata = create_test_piste_metadata();
    let json = serde_json::to_value(&metadata).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_piste_data_schema() {
    let schema = schema_for!(PisteData);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let data = create_test_piste_data();
    let json = serde_json::to_value(&data).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_piste_schema() {
    let schema = schema_for!(Piste);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let piste = create_test_piste();
    let json = serde_json::to_value(&piste).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_ski_area_metadata_schema() {
    let schema = schema_for!(SkiAreaMetadata);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let metadata = create_test_ski_area_metadata();
    let json = serde_json::to_value(&metadata).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_ski_area_schema() {
    let schema = schema_for!(SkiArea);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let ski_area = create_test_ski_area();
    let json = serde_json::to_value(&ski_area).unwrap();

    assert!(validator.validate(&json).is_ok());
}
