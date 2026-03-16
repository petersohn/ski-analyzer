use crate::json_schema::geo::{LineStringDef, PointDef, PolygonDef, RectDef};
use geo::{LineString, Point, Polygon};
use jsonschema::Validator;
use schemars::schema_for;

#[test]
fn validate_point_schema() {
    let schema = schema_for!(PointDef);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let point = Point::new(1.0, 2.0);
    let json = serde_json::to_value(&point).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_rect_schema() {
    let schema = schema_for!(RectDef);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let rect = geo::Rect::new(Point::new(1.0, 2.0), Point::new(3.0, 4.0));
    let json = serde_json::to_value(&rect).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_line_string_schema() {
    let schema = schema_for!(LineStringDef);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let line_string = LineString::from(vec![(0.0, 0.0), (1.0, 1.0)]);
    let json = serde_json::to_value(&line_string).unwrap();

    assert!(validator.validate(&json).is_ok());
}

#[test]
fn validate_polygon_schema() {
    let schema = schema_for!(PolygonDef);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let exterior = LineString::from(vec![
        (0.0, 0.0),
        (2.0, 0.0),
        (2.0, 2.0),
        (0.0, 2.0),
        (0.0, 0.0),
    ]);
    let interiors = vec![LineString::from(vec![
        (0.5, 0.5),
        (1.5, 0.5),
        (1.5, 1.5),
        (0.5, 1.5),
        (0.5, 0.5),
    ])];
    let polygon = Polygon::new(exterior, interiors);
    let json = serde_json::to_value(&polygon).unwrap();

    assert!(validator.validate(&json).is_ok());
}
