use super::segments::{get_segments, Segments};
use crate::test_util::{init, Init};

use geo::point;
use gpx::{Gpx, Track, TrackSegment, Waypoint};
use rstest::rstest;

fn segment(input: &[(f64, f64, Option<f64>)]) -> TrackSegment {
    let mut result = TrackSegment::new();
    result.points = input
        .iter()
        .map(|(x, y, h)| {
            let mut result = Waypoint::new(point! { x: *x, y: *y });
            result.hdop = *h;
            result
        })
        .collect();
    result
}

fn track(input: Vec<TrackSegment>) -> Track {
    let mut result = Track::new();
    result.segments = input;
    result
}

fn make_gpx(input: Vec<Track>) -> Gpx {
    let mut result = Gpx::default();
    result.tracks = input;
    result
}

fn ptrize(segments: &Segments) -> Vec<Vec<*const Waypoint>> {
    segments
        .iter()
        .map(|s| s.iter().map(|w| -> *const Waypoint { *w }).collect())
        .collect()
}

fn get_wp(
    gpx: &Gpx,
    track_id: usize,
    segment_id: usize,
    id: usize,
) -> &Waypoint {
    &gpx.tracks[track_id].segments[segment_id].points[id]
}

#[rstest]
fn multiple_tracks_and_segments(_init: Init) {
    let gpx = make_gpx(vec![
        track(vec![
            segment(&[
                (1.0, 1.0, Some(2.0)),
                (1.0, 2.0, Some(9.0)),
                (1.0, 3.0, Some(5.0)),
            ]),
            segment(&[
                (2.0, 1.0, Some(2.0)),
                (2.0, 2.0, None),
                (2.0, 3.0, Some(1.0)),
            ]),
        ]),
        track(vec![segment(&[
            (3.0, 1.0, Some(2.0)),
            (3.0, 2.0, Some(1.1)),
        ])]),
    ]);

    let actual = get_segments(&gpx);
    let expected = vec![
        vec![
            get_wp(&gpx, 0, 0, 0),
            get_wp(&gpx, 0, 0, 1),
            get_wp(&gpx, 0, 0, 2),
        ],
        vec![
            get_wp(&gpx, 0, 1, 0),
            get_wp(&gpx, 0, 1, 1),
            get_wp(&gpx, 0, 1, 2),
        ],
        vec![get_wp(&gpx, 1, 0, 0), get_wp(&gpx, 1, 0, 1)],
    ];
    assert_eq!(
        ptrize(&actual),
        ptrize(&expected),
        "actual={:#?}\nexpected={:#?}",
        actual,
        expected
    );
}

#[rstest]
fn bad_accuracy(_init: Init) {
    let gpx = make_gpx(vec![track(vec![
        segment(&[
            (1.0, 1.0, Some(2.0)),
            (1.0, 2.0, Some(9.0)),
            (1.0, 3.0, Some(5.0)),
        ]),
        segment(&[
            (2.0, 1.0, Some(2.0)),
            (2.0, 2.0, Some(1.0)),
            (2.0, 3.0, Some(5.0)),
            (2.0, 4.0, Some(11.0)),
            (2.0, 5.0, Some(20.0)),
            (2.0, 6.0, Some(10.5)),
            (2.0, 7.0, Some(22.0)),
            (2.0, 8.0, Some(11.0)),
            (2.0, 9.0, Some(9.0)),
            (2.0, 10.0, Some(1.0)),
            (2.0, 11.0, Some(4.0)),
            (2.0, 12.0, Some(8.0)),
        ]),
        segment(&[
            (3.0, 1.0, Some(2.0)),
            (3.0, 2.0, None),
            (3.0, 3.0, Some(1.0)),
        ]),
    ])]);

    let actual = get_segments(&gpx);
    let expected = vec![
        vec![
            get_wp(&gpx, 0, 0, 0),
            get_wp(&gpx, 0, 0, 1),
            get_wp(&gpx, 0, 0, 2),
        ],
        vec![
            get_wp(&gpx, 0, 1, 0),
            get_wp(&gpx, 0, 1, 1),
            get_wp(&gpx, 0, 1, 2),
        ],
        vec![
            get_wp(&gpx, 0, 1, 8),
            get_wp(&gpx, 0, 1, 9),
            get_wp(&gpx, 0, 1, 10),
            get_wp(&gpx, 0, 1, 11),
        ],
        vec![
            get_wp(&gpx, 0, 2, 0),
            get_wp(&gpx, 0, 2, 1),
            get_wp(&gpx, 0, 2, 2),
        ],
    ];
    assert_eq!(
        ptrize(&actual),
        ptrize(&expected),
        "actual={:#?}\nexpected={:#?}",
        actual,
        expected
    );
}
