use super::lift::parse_lift;
use super::{Lift, PointWithElevation};
use crate::osm_reader::{self as r, Osm3s};
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::test_util::{
    assert_eq_pretty, init, line, node, node_tags, way_tags, Init,
};

use geo::point;
use rstest::rstest;
use std::collections::HashMap;

#[rstest]
fn simple_minimal_info(_init: Init) {
    let doc = r::Document {
        osm3s: Osm3s::default(),
        elements: r::Elements {
            nodes: HashMap::from([
                (0, node(0.0, 0.0)),
                (1, node(1.0, 0.0)),
                (2, node(2.0, 0.0)),
                (3, node(3.0, 0.0)),
            ]),
            ways: HashMap::from([(
                101,
                way_tags(&[0, 1, 2, 3], &[("aerialway", "chair_lift")]),
            )]),
            relations: HashMap::new(),
        },
    };

    let actual =
        parse_lift(&doc, &101, doc.elements.ways.get(&101).unwrap()).unwrap();
    let expected = Some(Lift {
        unique_id: "101".to_string(),
        ref_: String::new(),
        name: "<unnamed chair_lift>".to_string(),
        type_: "chair_lift".to_string(),
        line: BoundedGeometry::new(line(&[
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
        ]))
        .unwrap(),
        stations: vec![
            PointWithElevation::new(point! {x: 0.0, y: 0.0}, 0),
            PointWithElevation::new(point! {x: 3.0, y: 0.0}, 0),
        ],
        can_go_reverse: false,
        can_disembark: false,
        lengths: Vec::new(),
    });
    assert_eq_pretty!(actual, expected);
}

#[rstest]
fn simple_more_info(_init: Init) {
    let doc = r::Document {
        osm3s: Osm3s::default(),
        elements: r::Elements {
            nodes: HashMap::from([
                (
                    0,
                    node_tags(
                        0.0,
                        1.0,
                        &[("aerialway", "station"), ("ele", "1000")],
                    ),
                ),
                (1, node(1.0, 1.0)),
                (2, node(2.0, 1.0)),
                (
                    3,
                    node_tags(
                        3.0,
                        1.0,
                        &[("aerialway", "station"), ("ele", "1400")],
                    ),
                ),
            ]),
            ways: HashMap::from([(
                101,
                way_tags(
                    &[0, 1, 2, 3],
                    &[("aerialway", "t-bar"), ("name", "Lift 1"), ("ref", "A")],
                ),
            )]),
            relations: HashMap::new(),
        },
    };

    let actual =
        parse_lift(&doc, &101, doc.elements.ways.get(&101).unwrap()).unwrap();
    let expected = Some(Lift {
        unique_id: "101".to_string(),
        ref_: "A".to_string(),
        name: "Lift 1".to_string(),
        type_: "t-bar".to_string(),
        line: BoundedGeometry::new(line(&[
            (0.0, 1.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (3.0, 1.0),
        ]))
        .unwrap(),
        stations: vec![
            PointWithElevation::new(point! {x: 0.0, y: 1.0}, 1000),
            PointWithElevation::new(point! {x: 3.0, y: 1.0}, 1400),
        ],
        can_go_reverse: false,
        can_disembark: true,
        lengths: Vec::new(),
    });
    assert_eq_pretty!(actual, expected);
}

#[rstest]
fn multiple_stations(_init: Init) {
    let doc = r::Document {
        osm3s: Osm3s::default(),
        elements: r::Elements {
            nodes: HashMap::from([
                (
                    0,
                    node_tags(
                        0.0,
                        2.0,
                        &[("aerialway", "station"), ("ele", "1000")],
                    ),
                ),
                (1, node(1.0, 2.0)),
                (2, node(2.0, 2.0)),
                (
                    3,
                    node_tags(
                        3.0,
                        2.0,
                        &[("aerialway", "station"), ("ele", "1200")],
                    ),
                ),
                (4, node(4.0, 2.0)),
                (
                    5,
                    node_tags(
                        5.0,
                        2.0,
                        &[("aerialway", "station"), ("ele", "1400")],
                    ),
                ),
                (
                    6,
                    node_tags(
                        6.0,
                        2.0,
                        &[("aerialway", "station"), ("ele", "1600")],
                    ),
                ),
                (7, node(7.0, 2.0)),
                (8, node(8.0, 2.0)),
                (9, node(9.0, 2.0)),
                (
                    10,
                    node_tags(
                        10.0,
                        2.0,
                        &[("aerialway", "station"), ("ele", "1800")],
                    ),
                ),
            ]),
            ways: HashMap::from([(
                101,
                way_tags(
                    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                    &[("aerialway", "gondola"), ("name", "Lift 2")],
                ),
            )]),
            relations: HashMap::new(),
        },
    };

    let actual =
        parse_lift(&doc, &101, doc.elements.ways.get(&101).unwrap()).unwrap();
    let expected = Some(Lift {
        unique_id: "101".to_string(),
        ref_: String::new(),
        name: "Lift 2".to_string(),
        type_: "gondola".to_string(),
        line: BoundedGeometry::new(line(&[
            (0.0, 2.0),
            (1.0, 2.0),
            (2.0, 2.0),
            (3.0, 2.0),
            (4.0, 2.0),
            (5.0, 2.0),
            (6.0, 2.0),
            (7.0, 2.0),
            (8.0, 2.0),
            (9.0, 2.0),
            (10.0, 2.0),
        ]))
        .unwrap(),
        stations: vec![
            PointWithElevation::new(point! {x: 0.0, y: 2.0}, 1000),
            PointWithElevation::new(point! {x: 3.0, y: 2.0}, 1200),
            PointWithElevation::new(point! {x: 5.0, y: 2.0}, 1400),
            PointWithElevation::new(point! {x: 6.0, y: 2.0}, 1600),
            PointWithElevation::new(point! {x: 10.0, y: 2.0}, 1800),
        ],
        can_go_reverse: true,
        can_disembark: false,
        lengths: Vec::new(),
    });
    assert_eq_pretty!(actual, expected);
}
