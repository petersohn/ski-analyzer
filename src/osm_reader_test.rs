use super::osm_reader as r;
use std::collections::HashMap;

#[test]
fn parse_document() {
    let json = r###"{
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
                    lat: 10.0,
                    lon: 20.0,
                    tags: HashMap::from([(
                        String::from("foo"),
                        String::from("foooo"),
                    )]),
                },
            ),
            (
                2,
                r::Node {
                    lat: 11.0,
                    lon: 21.0,
                    tags: HashMap::new(),
                },
            ),
            (
                3,
                r::Node {
                    lat: 12.0,
                    lon: 22.0,
                    tags: HashMap::new(),
                },
            ),
            (
                4,
                r::Node {
                    lat: 13.0,
                    lon: 23.0,
                    tags: HashMap::new(),
                },
            ),
            (
                5,
                r::Node {
                    lat: 14.0,
                    lon: 24.0,
                    tags: HashMap::new(),
                },
            ),
            (
                6,
                r::Node {
                    lat: 15.0,
                    lon: 25.0,
                    tags: HashMap::new(),
                },
            ),
            (
                7,
                r::Node {
                    lat: 16.0,
                    lon: 26.0,
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
                },
            ),
            (
                11,
                r::Way {
                    nodes: Vec::from([3, 4, 5]),
                    tags: HashMap::new(),
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
