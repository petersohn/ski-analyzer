use super::test_util::wp;
use super::{SegmentCoordinate, Segments};
use crate::assert_eq_pretty;

use gpx::Waypoint;
use rstest::{fixture, rstest};

#[fixture]
fn segmented() -> Segments {
    Segments::new(vec![
        vec![
            wp(1.0, 10.0, None),
            wp(2.0, 10.0, None),
            wp(3.0, 10.0, None),
            wp(4.0, 10.0, None),
        ],
        vec![wp(1.0, 20.0, None), wp(2.0, 20.0, None)],
        vec![
            wp(1.0, 30.0, None),
            wp(2.0, 30.0, None),
            wp(3.0, 30.0, None),
        ],
    ])
}

type FlattenedSegments = Vec<(SegmentCoordinate, Waypoint)>;

#[fixture]
fn flattened() -> FlattenedSegments {
    vec![
        ((0, 0), wp(1.0, 10.0, None)),
        ((0, 1), wp(2.0, 10.0, None)),
        ((0, 2), wp(3.0, 10.0, None)),
        ((0, 3), wp(4.0, 10.0, None)),
        ((1, 0), wp(1.0, 20.0, None)),
        ((1, 1), wp(2.0, 20.0, None)),
        ((2, 0), wp(1.0, 30.0, None)),
        ((2, 1), wp(2.0, 30.0, None)),
        ((2, 2), wp(3.0, 30.0, None)),
    ]
}

#[rstest]
fn iter(segmented: Segments, mut flattened: FlattenedSegments) {
    let actual_fwd: Vec<(SegmentCoordinate, Waypoint)> =
        segmented.iter().map(|(c, w)| (c, w.clone())).collect();
    assert_eq_pretty!(actual_fwd, flattened);

    flattened.reverse();
    let actual_rev: Vec<(SegmentCoordinate, Waypoint)> = segmented
        .iter()
        .rev()
        .map(|(c, w)| (c, w.clone()))
        .collect();
    assert_eq_pretty!(actual_rev, flattened);
}

#[rstest]
fn iter_from(segmented: Segments, flattened: FlattenedSegments) {
    let get_expected = |n| {
        flattened
            .iter()
            .skip(n)
            .map(|x| x.clone())
            .collect::<Vec<(SegmentCoordinate, Waypoint)>>()
    };
    let get_actual = |x, y| {
        segmented
            .iter_from((x, y))
            .map(|(c, w)| (c, w.clone()))
            .collect::<Vec<(SegmentCoordinate, Waypoint)>>()
    };

    assert_eq_pretty!(get_actual(0, 0), get_expected(0));
    assert_eq_pretty!(get_actual(0, 2), get_expected(2));
    assert_eq_pretty!(get_actual(0, 3), get_expected(3));
    assert_eq_pretty!(get_actual(0, 4), get_expected(4));
    assert_eq_pretty!(get_actual(0, 5), get_expected(4));
    assert_eq_pretty!(get_actual(1, 0), get_expected(4));
    assert_eq_pretty!(get_actual(1, 1), get_expected(5));
    assert_eq_pretty!(get_actual(2, 2), get_expected(8));
    assert_eq_pretty!(get_actual(2, 3), get_expected(9));
    assert_eq_pretty!(get_actual(2, 4), get_expected(9));
    assert_eq_pretty!(get_actual(3, 0), get_expected(9));
    assert_eq_pretty!(get_actual(3, 1), get_expected(9));
}

#[rstest]
fn iter_until(segmented: Segments, flattened: FlattenedSegments) {
    let get_expected = |n| {
        flattened
            .iter()
            .take(n)
            .map(|x| x.clone())
            .collect::<Vec<(SegmentCoordinate, Waypoint)>>()
    };
    let get_actual = |x, y| {
        segmented
            .iter_until((x, y))
            .map(|(c, w)| (c, w.clone()))
            .collect::<Vec<(SegmentCoordinate, Waypoint)>>()
    };

    assert_eq_pretty!(get_actual(0, 0), get_expected(0));
    assert_eq_pretty!(get_actual(0, 2), get_expected(2));
    assert_eq_pretty!(get_actual(0, 3), get_expected(3));
    assert_eq_pretty!(get_actual(0, 4), get_expected(4));
    assert_eq_pretty!(get_actual(0, 5), get_expected(4));
    assert_eq_pretty!(get_actual(1, 0), get_expected(4));
    assert_eq_pretty!(get_actual(1, 1), get_expected(5));
    assert_eq_pretty!(get_actual(2, 2), get_expected(8));
    assert_eq_pretty!(get_actual(2, 3), get_expected(9));
    assert_eq_pretty!(get_actual(2, 4), get_expected(9));
    assert_eq_pretty!(get_actual(3, 0), get_expected(9));
    assert_eq_pretty!(get_actual(3, 1), get_expected(9));
}

#[rstest]
fn iter_between(segmented: Segments, flattened: FlattenedSegments) {
    let get_expected = |from, to| {
        flattened
            .iter()
            .skip(from)
            .take(to - from)
            .map(|x| x.clone())
            .collect::<Vec<(SegmentCoordinate, Waypoint)>>()
    };
    let get_actual = |from, to| {
        segmented
            .iter_between(from, to)
            .map(|(c, w)| (c, w.clone()))
            .collect::<Vec<(SegmentCoordinate, Waypoint)>>()
    };

    assert_eq_pretty!(get_actual((0, 0), (3, 0)), get_expected(0, 9));
    assert_eq_pretty!(get_actual((0, 0), (1, 1)), get_expected(0, 5));
    assert_eq_pretty!(get_actual((1, 1), (3, 0)), get_expected(5, 9));
    assert_eq_pretty!(get_actual((0, 2), (2, 1)), get_expected(2, 7));
}

fn get_expected_part(
    segments: &Segments,
    coords: &[&[SegmentCoordinate]],
) -> Segments {
    Segments::new(
        coords
            .iter()
            .map(|s| {
                s.iter()
                    .map(|c| segments.get(*c).unwrap().clone())
                    .collect()
            })
            .collect(),
    )
}

#[rstest]
fn clone_part_empty(segmented: Segments) {
    assert_eq_pretty!(
        segmented.clone_part((0, 0), (0, 0)),
        Segments::new(vec![])
    );
    assert_eq_pretty!(
        segmented.clone_part((0, 2), (0, 2)),
        Segments::new(vec![])
    );
    assert_eq_pretty!(
        segmented.clone_part((1, 1), (1, 1)),
        Segments::new(vec![])
    );
}

#[rstest]
fn clone_part_within_segment(segmented: Segments) {
    assert_eq_pretty!(
        segmented.clone_part((0, 1), (0, 3)),
        get_expected_part(&segmented, &[&[(0, 1), (0, 2)]])
    );
}

#[rstest]
fn clone_part_between_segments(segmented: Segments) {
    assert_eq_pretty!(
        segmented.clone_part((0, 2), (2, 1)),
        get_expected_part(
            &segmented,
            &[&[(0, 2), (0, 3)], &[(1, 0), (1, 1)], &[(2, 0)]]
        )
    );
}

#[rstest]
fn clone_part_whole_segment(segmented: Segments) {
    assert_eq_pretty!(
        segmented.clone_part((1, 0), (2, 0)),
        get_expected_part(&segmented, &[&[(1, 0), (1, 1)]])
    );
}

#[rstest]
fn clone_part_beginning(segmented: Segments) {
    assert_eq_pretty!(
        segmented.clone_part((0, 0), (1, 1)),
        get_expected_part(
            &segmented,
            &[&[(0, 0), (0, 1), (0, 2), (0, 3)], &[(1, 0)]]
        )
    );
}

#[rstest]
fn clone_part_end(segmented: Segments) {
    assert_eq_pretty!(
        segmented.clone_part((1, 1), segmented.end_coord()),
        get_expected_part(&segmented, &[&[(1, 1)], &[(2, 0), (2, 1), (2, 2)]])
    );
}

fn test_split_end(
    segments: Segments,
    coord: SegmentCoordinate,
    input_end: SegmentCoordinate,
    output_begin: SegmentCoordinate,
) {
    let mut input = segments.clone();
    let output = input.split_end(coord);

    assert_eq_pretty!(
        input,
        segments.clone_part(segments.begin_coord(), input_end)
    );
    assert_eq_pretty!(
        output,
        segments.clone_part(output_begin, segments.end_coord())
    );
}

#[rstest]
fn split_end_at_segment_begin(segmented: Segments) {
    test_split_end(segmented, (1, 0), (1, 0), (1, 0));
}

#[rstest]
fn split_end_at_segment_middle(segmented: Segments) {
    test_split_end(segmented, (1, 1), (2, 0), (1, 1));
}

fn wp_with_elevation(x: f64, y: f64, elevation: f64) -> Waypoint {
    let mut result = Waypoint::new(geo::point!(x: x, y: y));
    result.elevation = Some(elevation);
    result
}

#[test]
fn test_segments_serialize_deserialize_roundtrip() {
    let original = Segments::new(vec![
        vec![
            wp_with_elevation(8.0, 47.0, 1000.0),
            wp_with_elevation(8.1, 47.1, 1500.0),
        ],
        vec![wp(8.2, 47.2, None)],
    ]);

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Segments = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.0.len(), 2);
    assert_eq!(deserialized.0[0].len(), 2);
    assert_eq!(deserialized.0[1].len(), 1);

    assert!((deserialized.0[0][0].point().x() - 8.0).abs() < 0.001);
    assert!((deserialized.0[0][0].elevation.unwrap() - 1000.0).abs() < 0.001);
}

#[test]
fn test_empty_segments() {
    let original = super::Segments::new(vec![]);
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: super::Segments = serde_json::from_str(&json).unwrap();
    assert!(deserialized.0.is_empty());
}
