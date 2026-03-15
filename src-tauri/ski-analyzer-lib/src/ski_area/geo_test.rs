use geo::{LineString, Point, Polygon, Rect};

#[test]
fn test_point_serialization() {
    let point = Point::new(1.5, 2.5);
    let json = serde_json::to_string(&point).unwrap();
    println!("Point JSON: {}", json);

    let parsed: Point<f64> = serde_json::from_str(&json).unwrap();
    assert_eq!(point, parsed);
}

#[test]
fn test_point_serialization_format() {
    let point = Point::new(1.5, 2.5);
    let json = serde_json::to_value(&point).unwrap();

    assert!(json.is_object());
    let obj = json.as_object().unwrap();
    assert!(obj.contains_key("x"));
    assert!(obj.contains_key("y"));
    assert_eq!(obj["x"], serde_json::json!(1.5));
    assert_eq!(obj["y"], serde_json::json!(2.5));
}

#[test]
fn test_rect_serialization() {
    let rect = Rect::new(Point::new(1.0, 2.0), Point::new(3.0, 4.0));
    let json = serde_json::to_string(&rect).unwrap();
    println!("Rect JSON: {}", json);

    let parsed: Rect<f64> = serde_json::from_str(&json).unwrap();
    assert_eq!(rect, parsed);
}

#[test]
fn test_rect_serialization_format() {
    let rect = Rect::new(Point::new(1.0, 2.0), Point::new(3.0, 4.0));
    let json = serde_json::to_value(&rect).unwrap();

    assert!(json.is_object());
    let obj = json.as_object().unwrap();
    assert!(obj.contains_key("min"));
    assert!(obj.contains_key("max"));

    let min = &obj["min"];
    assert_eq!(min["x"], serde_json::json!(1.0));
    assert_eq!(min["y"], serde_json::json!(2.0));

    let max = &obj["max"];
    assert_eq!(max["x"], serde_json::json!(3.0));
    assert_eq!(max["y"], serde_json::json!(4.0));
}

#[test]
fn test_line_string_serialization() {
    let coords = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
    let line_string = LineString::from(coords);
    let json = serde_json::to_string(&line_string).unwrap();
    println!("LineString JSON: {}", json);

    let parsed: LineString<f64> = serde_json::from_str(&json).unwrap();
    assert_eq!(line_string, parsed);
}

#[test]
fn test_line_string_serialization_format() {
    let coords = vec![(0.0, 0.0), (1.0, 1.0)];
    let line_string = LineString::from(coords);
    let json = serde_json::to_value(&line_string).unwrap();

    assert!(json.is_array());
    let arr = json.as_array().unwrap();

    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0], serde_json::json!({"x": 0.0, "y": 0.0}));
    assert_eq!(arr[1], serde_json::json!({"x": 1.0, "y": 1.0}));
}

#[test]
fn test_polygon_serialization() {
    let exterior = LineString::from(vec![
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
        (0.0, 0.0),
    ]);
    let interiors = vec![LineString::from(vec![
        (0.2, 0.2),
        (0.8, 0.2),
        (0.8, 0.8),
        (0.2, 0.8),
        (0.2, 0.2),
    ])];
    let polygon = Polygon::new(exterior, interiors);
    let json = serde_json::to_string(&polygon).unwrap();
    println!("Polygon JSON: {}", json);

    let parsed: Polygon<f64> = serde_json::from_str(&json).unwrap();
    assert_eq!(polygon, parsed);
}

#[test]
fn test_polygon_serialization_format() {
    let exterior = LineString::from(vec![
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
        (0.0, 0.0),
    ]);
    let polygon = Polygon::new(exterior, vec![]);
    let json = serde_json::to_value(&polygon).unwrap();

    assert!(json.is_object());
    let obj = json.as_object().unwrap();
    assert!(obj.contains_key("exterior"));
    assert!(obj.contains_key("interiors"));

    let exterior_ring = &obj["exterior"];
    assert!(exterior_ring.is_array());

    let coords = exterior_ring.as_array().unwrap();
    assert_eq!(coords.len(), 5);
    assert_eq!(coords[0], serde_json::json!({"x": 0.0, "y": 0.0}));
    assert_eq!(coords[1], serde_json::json!({"x": 1.0, "y": 0.0}));
    assert_eq!(coords[2], serde_json::json!({"x": 1.0, "y": 1.0}));
    assert_eq!(coords[3], serde_json::json!({"x": 0.0, "y": 1.0}));
    assert_eq!(coords[4], serde_json::json!({"x": 0.0, "y": 0.0}));

    let interiors = &obj["interiors"];
    assert!(interiors.is_array());
    assert_eq!(interiors.as_array().unwrap().len(), 0);
}

#[test]
fn test_polygon_with_holes_serialization_format() {
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

    assert!(json.is_object());
    let obj = json.as_object().unwrap();

    let exterior_ring = &obj["exterior"];
    assert_eq!(exterior_ring.as_array().unwrap().len(), 5);

    let interiors_arr = &obj["interiors"];
    assert!(interiors_arr.is_array());
    let interiors = interiors_arr.as_array().unwrap();
    assert_eq!(interiors.len(), 1);
    assert_eq!(interiors[0].as_array().unwrap().len(), 5);
}
