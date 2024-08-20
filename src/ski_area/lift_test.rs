use super::lift::parse_lift;
use super::{BoundedGeometry, Lift, PointWithElevation};
use crate::osm_reader as r;
use crate::test_util::{init, line, node, node_tags, way_tags, Init};

use geo::point;
use rstest::rstest;
use std::collections::HashMap;

#[rstest]
fn simple_no_midstations(_init: Init) {
    let doc = r::Document {
        elements: r::Elements {
            nodes: HashMap::from([
                (0, node(0.0, 0.0)),
                (1, node(1.0, 0.0)),
                (2, node(2.0, 0.0)),
                (3, node(3.0, 0.0)),
            ]),
            ways: HashMap::from([(
                101,
                way_tags(
                    &[0, 1, 2, 3],
                    &[
                        ("aerialway", "chair_lift"),
                        ("name", "Lift1"),
                        ("ref", "A"),
                    ],
                ),
            )]),
            relations: HashMap::new(),
        },
    };

    let actual =
        parse_lift(&doc, &101, doc.elements.ways.get(&101).unwrap()).unwrap();
    let expected = Some(Lift {
        ref_: "A".to_string(),
        name: "Lift1".to_string(),
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
    });
    assert_eq!(actual, expected);
}
