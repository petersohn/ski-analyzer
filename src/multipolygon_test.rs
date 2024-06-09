use super::multipolygon::parse_multipolygon;
use super::osm_reader as r;
use geo::{coord, LineString, MultiPolygon, Polygon};
use std::collections::HashMap;

fn is_equal_l(lhs: &LineString, rhs: &LineString) -> bool {
    if !lhs.is_closed() || !rhs.is_closed() || lhs.0.len() != rhs.0.len() {
        return false;
    }

    if lhs.0 == rhs.0 {
        return true;
    }

    let mut rot = rhs.0.clone();
    for _ in 0..rhs.0.len() - 2 {
        rot.pop();
        rot.rotate_right(1);
        rot.push(rot[0]);
        if lhs.0 == rot {
            return true;
        }
    }

    false
}

fn is_equal_set<T, Op>(lhs: &[T], rhs: &[T], op: Op) -> bool
where
    T: Clone,
    Op: Fn(&T, &T) -> bool,
{
    if lhs.len() != rhs.len() {
        return false;
    }

    let mut rhss = rhs.to_vec();

    for l in lhs {
        for i in 0..rhss.len() {
            if op(&l, &rhss[i]) {
                rhss.remove(i);
                break;
            }
        }
    }

    rhss.is_empty()
}

fn is_equal_p(lhs: &Polygon, rhs: &Polygon) -> bool {
    if !is_equal_l(lhs.exterior(), rhs.exterior()) {
        return false;
    }

    is_equal_set(lhs.interiors(), rhs.interiors(), is_equal_l)
}

fn is_equal(lhs: &MultiPolygon, rhs: &MultiPolygon) -> bool {
    is_equal_set(&lhs.0, &rhs.0, is_equal_p)
}

fn node(x: f64, y: f64) -> r::Node {
    r::Node {
        lat: y,
        lon: x,
        tags: HashMap::new(),
    }
}

fn way(ids: &[u64]) -> r::Way {
    r::Way {
        nodes: Vec::from(ids),
        tags: HashMap::new(),
    }
}

fn mp(outers: &[u64], inners: &[u64]) -> r::Relation {
    let ways = outers
        .iter()
        .map(|id| r::RelationMember {
            ref_: id.clone(),
            role: String::from("outer"),
        })
        .chain(inners.iter().map(|id| r::RelationMember {
            ref_: id.clone(),
            role: String::from("inner"),
        }))
        .collect();
    r::Relation {
        members: r::RelationMembers {
            nodes: Vec::new(),
            ways,
        },
        tags: HashMap::from([(
            String::from("type"),
            String::from("multipolygon"),
        )]),
    }
}

fn line(points: &[(f64, f64)]) -> LineString {
    LineString(
        points
            .iter()
            .map(|(x, y)| coord! { x: x.clone(), y: y.clone() })
            .collect(),
    )
}

// https://wiki.openstreetmap.org/wiki/Relation:multipolygon#One_outer_and_one_inner_ring
#[test]
fn one_outer_and_one_inner_ring() {
    let doc = r::Document {
        elements: r::Elements {
            nodes: HashMap::from([
                (0, node(8.0, 2.0)),
                (1, node(12.0, 4.0)),
                (2, node(13.0, 8.0)),
                (3, node(8.0, 11.0)),
                (4, node(5.0, 7.0)),
                (10, node(8.0, 5.0)),
                (11, node(10.0, 6.0)),
                (12, node(9.0, 8.0)),
                (13, node(7.0, 7.0)),
            ]),
            ways: HashMap::from([
                (100, way(&[0, 1, 2, 3, 4, 0])),
                (101, way(&[10, 11, 12, 13, 10])),
            ]),
            relations: HashMap::from([(200, mp(&[100], &[101]))]),
        },
    };

    let actual =
        parse_multipolygon(&doc, doc.elements.relations.get(&200).unwrap())
            .unwrap();
    let expected = MultiPolygon(vec![Polygon::new(
        line(&[
            (8.0, 2.0),
            (12.0, 4.0),
            (13.0, 8.0),
            (8.0, 11.0),
            (5.0, 7.0),
            (8.0, 2.0),
        ]),
        vec![line(&[
            (8.0, 5.0),
            (10.0, 6.0),
            (9.0, 8.0),
            (7.0, 7.0),
            (8.0, 5.0),
        ])],
    )]);

    assert!(
        is_equal(&actual, &expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

// https://wiki.openstreetmap.org/wiki/Relation:multipolygon#One_outer_and_two_inner_rings
#[test]
fn one_outer_and_two_inner_rings() {
    let doc = r::Document {
        elements: r::Elements {
            nodes: HashMap::from([
                (0, node(8.0, 2.0)),
                (1, node(12.0, 4.0)),
                (2, node(13.0, 8.0)),
                (3, node(8.0, 11.0)),
                (4, node(5.0, 7.0)),
                (10, node(8.0, 3.0)),
                (11, node(10.0, 4.0)),
                (12, node(9.0, 6.0)),
                (13, node(7.0, 5.0)),
                (20, node(9.0, 7.0)),
                (21, node(11.0, 7.0)),
                (22, node(10.0, 9.0)),
                (23, node(8.0, 8.0)),
            ]),
            ways: HashMap::from([
                (100, way(&[0, 1, 2, 3, 4, 0])),
                (101, way(&[10, 11, 12, 13, 10])),
                (102, way(&[20, 21, 22, 23, 20])),
            ]),
            relations: HashMap::from([(200, mp(&[100], &[101, 102]))]),
        },
    };

    let actual =
        parse_multipolygon(&doc, doc.elements.relations.get(&200).unwrap())
            .unwrap();
    let expected = MultiPolygon(vec![Polygon::new(
        line(&[
            (8.0, 2.0),
            (12.0, 4.0),
            (13.0, 8.0),
            (8.0, 11.0),
            (5.0, 7.0),
            (8.0, 2.0),
        ]),
        vec![
            line(&[
                (8.0, 3.0),
                (10.0, 4.0),
                (9.0, 6.0),
                (7.0, 5.0),
                (8.0, 3.0),
            ]),
            line(&[
                (9.0, 7.0),
                (11.0, 7.0),
                (10.0, 9.0),
                (8.0, 8.0),
                (9.0, 7.0),
            ]),
        ],
    )]);

    assert!(
        is_equal(&actual, &expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

// https://wiki.openstreetmap.org/wiki/Relation:multipolygon#Multiple_ways_forming_a_ring
#[test]
fn multiple_ways_forming_a_ring() {
    let doc = r::Document {
        elements: r::Elements {
            nodes: HashMap::from([
                (0, node(8.0, 2.0)),
                (1, node(12.0, 4.0)),
                (2, node(13.0, 8.0)),
                (3, node(8.0, 11.0)),
                (4, node(5.0, 7.0)),
                (10, node(8.0, 5.0)),
                (11, node(10.0, 6.0)),
                (12, node(9.0, 8.0)),
                (13, node(7.0, 7.0)),
            ]),
            ways: HashMap::from([
                (100, way(&[4, 0, 1])),
                (101, way(&[1, 2, 3, 4])),
                (102, way(&[10, 11, 12, 13, 10])),
            ]),
            relations: HashMap::from([(200, mp(&[100, 101], &[102]))]),
        },
    };

    let actual =
        parse_multipolygon(&doc, doc.elements.relations.get(&200).unwrap())
            .unwrap();
    let expected = MultiPolygon(vec![Polygon::new(
        line(&[
            (8.0, 2.0),
            (12.0, 4.0),
            (13.0, 8.0),
            (8.0, 11.0),
            (5.0, 7.0),
            (8.0, 2.0),
        ]),
        vec![line(&[
            (8.0, 5.0),
            (10.0, 6.0),
            (9.0, 8.0),
            (7.0, 7.0),
            (8.0, 5.0),
        ])],
    )]);

    assert!(
        is_equal(&actual, &expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

// https://wiki.openstreetmap.org/wiki/Relation:multipolygon#Two_disjunct_outer_rings
#[test]
fn two_disjunct_outer_rings() {
    let doc = r::Document {
        elements: r::Elements {
            nodes: HashMap::from([
                (0, node(4.0, 2.0)),
                (1, node(8.0, 4.0)),
                (2, node(9.0, 8.0)),
                (3, node(4.0, 11.0)),
                (4, node(1.0, 7.0)),
                (10, node(11.0, 2.0)),
                (11, node(15.0, 5.0)),
                (12, node(12.0, 11.0)),
                (13, node(10.0, 8.0)),
                (14, node(9.0, 4.0)),
            ]),
            ways: HashMap::from([
                (100, way(&[0, 1, 2, 3, 4, 0])),
                (101, way(&[10, 11, 12, 13, 14, 10])),
            ]),
            relations: HashMap::from([(200, mp(&[100, 101], &[]))]),
        },
    };

    let actual =
        parse_multipolygon(&doc, doc.elements.relations.get(&200).unwrap())
            .unwrap();
    let expected = MultiPolygon(vec![
        Polygon::new(
            line(&[
                (4.0, 2.0),
                (8.0, 4.0),
                (9.0, 8.0),
                (4.0, 11.0),
                (1.0, 7.0),
                (4.0, 2.0),
            ]),
            vec![],
        ),
        Polygon::new(
            line(&[
                (11.0, 2.0),
                (15.0, 5.0),
                (12.0, 11.0),
                (10.0, 8.0),
                (9.0, 4.0),
                (11.0, 2.0),
            ]),
            vec![],
        ),
    ]);

    assert!(
        is_equal(&actual, &expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

// https://wiki.openstreetmap.org/wiki/Relation:multipolygon#Two_disjunct_outer_rings_and_multiple_ways_forming_a_ring
#[test]
fn two_disjunct_outer_rings_and_multiple_ways_forming_a_ring() {
    let doc = r::Document {
        elements: r::Elements {
            nodes: HashMap::from([
                (0, node(4.0, 2.0)),
                (1, node(8.0, 4.0)),
                (2, node(9.0, 8.0)),
                (3, node(4.0, 11.0)),
                (4, node(1.0, 7.0)),
                (10, node(11.0, 2.0)),
                (11, node(15.0, 5.0)),
                (12, node(12.0, 11.0)),
                (13, node(10.0, 8.0)),
                (14, node(9.0, 4.0)),
                (20, node(3.0, 6.0)),
                (21, node(6.0, 4.0)),
                (22, node(7.0, 6.0)),
                (23, node(6.0, 9.0)),
                (24, node(4.0, 9.0)),
                (30, node(10.0, 4.0)),
                (31, node(12.0, 4.0)),
                (32, node(14.0, 6.0)),
                (33, node(12.0, 8.0)),
                (34, node(10.0, 7.0)),
            ]),
            ways: HashMap::from([
                (100, way(&[0, 1, 2, 3, 4, 0])),
                (101, way(&[20, 21, 22])),
                (102, way(&[22, 23, 24, 20])),
                (103, way(&[10, 11, 12, 13, 14, 10])),
                (104, way(&[30, 31, 32, 33, 34, 30])),
            ]),
            relations: HashMap::from([(
                200,
                mp(&[100, 103], &[101, 102, 104]),
            )]),
        },
    };

    let actual =
        parse_multipolygon(&doc, doc.elements.relations.get(&200).unwrap())
            .unwrap();
    let expected = MultiPolygon(vec![
        Polygon::new(
            line(&[
                (4.0, 2.0),
                (8.0, 4.0),
                (9.0, 8.0),
                (4.0, 11.0),
                (1.0, 7.0),
                (4.0, 2.0),
            ]),
            vec![line(&[
                (3.0, 6.0),
                (6.0, 4.0),
                (7.0, 6.0),
                (6.0, 9.0),
                (4.0, 9.0),
                (3.0, 6.0),
            ])],
        ),
        Polygon::new(
            line(&[
                (11.0, 2.0),
                (15.0, 5.0),
                (12.0, 11.0),
                (10.0, 8.0),
                (9.0, 4.0),
                (11.0, 2.0),
            ]),
            vec![line(&[
                (10.0, 4.0),
                (12.0, 4.0),
                (14.0, 6.0),
                (12.0, 8.0),
                (10.0, 7.0),
                (10.0, 4.0),
            ])],
        ),
    ]);

    assert!(
        is_equal(&actual, &expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

// https://wiki.openstreetmap.org/wiki/Relation:multipolygon#Island_within_a_hole
#[test]
fn island_within_a_hole() {
    let doc = r::Document {
        elements: r::Elements {
            nodes: HashMap::from([
                (0, node(6.0, 1.0)),
                (1, node(13.0, 2.0)),
                (2, node(14.0, 9.0)),
                (3, node(9.0, 12.0)),
                (4, node(2.0, 7.0)),
                (10, node(8.0, 2.0)),
                (11, node(12.0, 6.0)),
                (12, node(10.0, 11.0)),
                (13, node(5.0, 7.0)),
                (20, node(8.0, 6.0)),
                (21, node(10.0, 6.0)),
                (22, node(9.0, 8.0)),
                (23, node(7.0, 7.0)),
            ]),
            ways: HashMap::from([
                (100, way(&[0, 1, 2, 3, 4, 0])),
                (101, way(&[10, 11, 12, 13, 10])),
                (102, way(&[20, 21, 22, 23, 20])),
            ]),
            relations: HashMap::from([(200, mp(&[100, 102], &[101]))]),
        },
    };

    let actual =
        parse_multipolygon(&doc, doc.elements.relations.get(&200).unwrap())
            .unwrap();
    let expected = MultiPolygon(vec![
        Polygon::new(
            line(&[
                (6.0, 1.0),
                (13.0, 2.0),
                (14.0, 9.0),
                (9.0, 12.0),
                (2.0, 7.0),
                (6.0, 1.0),
            ]),
            vec![line(&[
                (8.0, 2.0),
                (12.0, 6.0),
                (10.0, 11.0),
                (5.0, 7.0),
                (8.0, 2.0),
            ])],
        ),
        Polygon::new(
            line(&[
                (8.0, 6.0),
                (10.0, 6.0),
                (9.0, 8.0),
                (7.0, 7.0),
                (8.0, 6.0),
            ]),
            vec![],
        ),
    ]);

    assert!(
        is_equal(&actual, &expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}
