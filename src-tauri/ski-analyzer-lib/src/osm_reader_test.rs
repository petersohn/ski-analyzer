use geo::{coord, LineString};

use super::osm_reader as r;
use std::collections::HashMap;

#[test]
fn parse_document() {
    let json = r###"{
    "osm3s": {
      "timestamp_osm_base": "2024-10-02T18:07:14Z",
      "timestamp_areas_base": "2024-09-16T09:46:45Z",
      "copyright": "Whatever"
    },
    "elements": [
      {
        "type": "node",
        "id": 1,
        "lat": 10.0,
        "lon": 20.0,
        "tags": {
          "foo": "foooo"
        }
      },
      {
        "type": "node",
        "id": 2,
        "lat": 11.0,
        "lon": 21.0
      },
      {
        "type": "node",
        "id": 3,
        "lat": 12.0,
        "lon": 22.0
      },
      {
        "type": "node",
        "id": 4,
        "lat": 13.0,
        "lon": 23.0
      },
      {
        "type": "node",
        "id": 5,
        "lat": 14.0,
        "lon": 24.0
      },
      {
        "type": "node",
        "id": 6,
        "lat": 15.0,
        "lon": 25.0
      },
      {
        "type": "node",
        "id": 7,
        "lat": 16.0,
        "lon": 26.0,
        "tags": {
          "foo": "2",
          "foobar": "foobarbaz"
        }
      },
      {
        "type": "way",
        "id": 10,
        "nodes": [1, 2, 5],
        "tags": {
          "bar": "baz"
        }
      },
      {
        "type": "way",
        "id": 11,
        "nodes": [3, 4, 5]
      },
      {
        "type": "relation",
        "id": 20,
        "members": [
          {
            "type": "node",
            "ref": 1,
            "role": "foorole"
          },
          {
            "type": "node",
            "ref": 6,
            "role": "barrole"
          },
          {
            "type": "way",
            "ref": 10,
            "role": "bazrole"
          }
        ],
        "tags": {
          "foo": "1",
          "bar": "foobar"
        }
      }
    ]
}"###;
    let document = r::Document::parse(json.as_bytes()).unwrap();
    let expected_elements = r::Elements {
        nodes: HashMap::from([
            (
                1,
                r::Node {
                    coordinate: r::Coordinate {
                        lat: 10.0,
                        lon: 20.0,
                    },
                    tags: HashMap::from([(
                        String::from("foo"),
                        String::from("foooo"),
                    )]),
                },
            ),
            (
                2,
                r::Node {
                    coordinate: r::Coordinate {
                        lat: 11.0,
                        lon: 21.0,
                    },
                    tags: HashMap::new(),
                },
            ),
            (
                3,
                r::Node {
                    coordinate: r::Coordinate {
                        lat: 12.0,
                        lon: 22.0,
                    },
                    tags: HashMap::new(),
                },
            ),
            (
                4,
                r::Node {
                    coordinate: r::Coordinate {
                        lat: 13.0,
                        lon: 23.0,
                    },
                    tags: HashMap::new(),
                },
            ),
            (
                5,
                r::Node {
                    coordinate: r::Coordinate {
                        lat: 14.0,
                        lon: 24.0,
                    },
                    tags: HashMap::new(),
                },
            ),
            (
                6,
                r::Node {
                    coordinate: r::Coordinate {
                        lat: 15.0,
                        lon: 25.0,
                    },
                    tags: HashMap::new(),
                },
            ),
            (
                7,
                r::Node {
                    coordinate: r::Coordinate {
                        lat: 16.0,
                        lon: 26.0,
                    },
                    tags: HashMap::from([
                        (String::from("foo"), String::from("2")),
                        (String::from("foobar"), String::from("foobarbaz")),
                    ]),
                },
            ),
        ]),
        ways: HashMap::from([
            (
                10,
                r::Way {
                    nodes: Vec::from([1, 2, 5]),
                    tags: HashMap::from([(
                        String::from("bar"),
                        String::from("baz"),
                    )]),
                    geometry: vec![],
                },
            ),
            (
                11,
                r::Way {
                    nodes: Vec::from([3, 4, 5]),
                    tags: HashMap::new(),
                    geometry: vec![],
                },
            ),
        ]),
        relations: HashMap::from([(
            20,
            r::Relation {
                members: r::RelationMembers {
                    nodes: Vec::from([
                        r::RelationMember {
                            ref_: 1,
                            role: String::from("foorole"),
                        },
                        r::RelationMember {
                            ref_: 6,
                            role: String::from("barrole"),
                        },
                    ]),
                    ways: Vec::from([r::RelationMember {
                        ref_: 10,
                        role: String::from("bazrole"),
                    }]),
                },
                tags: HashMap::from([
                    (String::from("foo"), String::from("1")),
                    (String::from("bar"), String::from("foobar")),
                ]),
            },
        )]),
    };

    assert_eq!(document.elements, expected_elements);
}

#[test]
fn parse_way_with_geometry() {
    let json = r###"{
    "osm3s": {
      "timestamp_osm_base": "2024-10-02T18:07:14Z",
      "timestamp_areas_base": "2024-09-16T09:46:45Z",
      "copyright": "Whatever"
    },
    "elements": [
      {
        "type": "way",
        "id": 10,
        "nodes": [1, 2, 5],
        "geometry": [
          { "lat": 10.0, "lon": -1.0 },
          { "lat": 11.0, "lon": -1.2 },
          { "lat": 12.0, "lon": 1.0 }
        ],
        "tags": {
          "bar": "baz"
        }
      }
    ]
}"###;
    let document = r::Document::parse(json.as_bytes()).unwrap();
    let expected_elements = r::Elements {
        nodes: HashMap::new(),
        ways: HashMap::from([(
            10,
            r::Way {
                nodes: Vec::from([1, 2, 5]),
                tags: HashMap::from([(
                    String::from("bar"),
                    String::from("baz"),
                )]),
                geometry: vec![
                    r::Coordinate {
                        lat: 10.0,
                        lon: -1.0,
                    },
                    r::Coordinate {
                        lat: 11.0,
                        lon: -1.2,
                    },
                    r::Coordinate {
                        lat: 12.0,
                        lon: 1.0,
                    },
                ],
            },
        )]),
        relations: HashMap::new(),
    };

    assert_eq!(document.elements, expected_elements);

    let expected_line = LineString::new(vec![
        coord! { x: -1.0, y: 10.0 },
        coord! { x: -1.2, y: 11.0 },
        coord! { x: 1.0, y: 12.0 },
    ]);

    assert_eq!(
        document.elements.ways[&10].geom_to_line_string(),
        expected_line
    );
}
