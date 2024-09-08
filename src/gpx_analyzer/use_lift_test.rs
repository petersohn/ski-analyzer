use super::test_util::{ptrize, SegmentsPtr};
use super::use_lift::find_lift_usage;
use super::Segments;
use super::{Activity, ActivityType, LiftEnd, UseLift};
use crate::ski_area::{BoundedGeometry, Lift, PointWithElevation, SkiArea};
use crate::test_util::{init, Init};

use geo::{coord, point, LineString};
use gpx::{Gpx, Track, TrackSegment, Waypoint};
use rstest::{fixture, rstest};
use time::OffsetDateTime;

fn line(input: &[(f64, f64)]) -> LineString {
    LineString::new(
        input.iter().map(|(x, y)| coord! { x: *x, y: *y }).collect(),
    )
}

fn lift(
    name: String,
    line_: LineString,
    midstations: &[usize],
    can_go_reverse: bool,
    can_disembark: bool,
) -> Lift {
    let stations = [0]
        .iter()
        .chain(midstations.iter())
        .chain([line_.0.len() - 1].iter())
        .map(|i| PointWithElevation {
            point: line_[*i].into(),
            elevation: 0,
        })
        .collect();
    let mut line = BoundedGeometry::new(line_).unwrap();
    line.expand(10.0);
    Lift {
        ref_: String::new(),
        name,
        type_: String::new(),
        line,
        stations,
        can_go_reverse,
        can_disembark,
    }
}

fn ski_area(lifts: Vec<Lift>) -> SkiArea {
    SkiArea {
        name: String::new(),
        lifts,
        pistes: Vec::new(),
    }
}

fn segment(input: &[(f64, f64, Option<OffsetDateTime>)]) -> TrackSegment {
    let mut result = TrackSegment::new();
    result.points = input
        .iter()
        .map(|(x, y, t)| {
            let mut result = Waypoint::new(point! { x: *x, y: *y });
            result.time = t.map(|tt| tt.into());
            result
        })
        .collect();
    result
}

fn make_gpx(input: Vec<TrackSegment>) -> Gpx {
    let mut track = Track::new();
    track.segments = input;
    let mut result = Gpx::default();
    result.tracks = vec![track];
    result
}

fn get_segments<'g>(gpx: &'g Gpx) -> Segments<'g> {
    gpx.tracks
        .iter()
        .map(|t| t.segments.iter().map(|s| s.points.iter().collect()))
        .flatten()
        .collect()
}

#[derive(PartialEq, Eq)]
pub struct UseLiftPtr {
    lift: *const Lift,
    begin_time: Option<OffsetDateTime>,
    end_time: Option<OffsetDateTime>,
    begin_station: LiftEnd,
    end_station: LiftEnd,
    is_reverse: bool,
}

type ComparableActivity = (Option<UseLiftPtr>, SegmentsPtr);

fn ptrize_activities(input: &[Activity]) -> Vec<ComparableActivity> {
    input
        .iter()
        .map(|a| {
            let type_ = match &a.type_ {
                ActivityType::UseLift(u) => Some(UseLiftPtr {
                    lift: u.lift,
                    begin_time: u.begin_time,
                    end_time: u.end_time,
                    begin_station: u.begin_station,
                    end_station: u.end_station,
                    is_reverse: u.is_reverse,
                }),
                _ => None,
            };
            (type_, ptrize(&a.route))
        })
        .collect()
}

fn get_segment_part<'g>(
    segments: &Segments<'g>,
    begin: (usize, usize),
    end: (usize, usize),
) -> Segments<'g> {
    if begin.0 == end.0 {
        return vec![segments[begin.0].get(begin.1..end.1).unwrap().into()];
    }
    let mut result = Vec::new();
    result.reserve(end.0 - begin.0 + 1);
    result.push(segments[begin.0].get(begin.1..).unwrap().into());
    for i in (begin.0 + 1)..end.0 {
        result.push(segments[i].clone());
    }
    result.push(segments[end.0].get(0..end.1).unwrap().into());
    result
}

#[fixture]
fn line00() -> LineString {
    line(&[
        (6.6532147, 45.3865145),
        (6.6531819, 45.3855226),
        (6.6531714, 45.3852169),
        (6.6529913, 45.3799574),
        (6.6528696, 45.3729557),
    ])
}

#[fixture]
fn line01() -> LineString {
    line(&[
        (6.6531285, 45.3865073),
        (6.6530753, 45.3855296),
        (6.6530649, 45.3852277),
        (6.6528845, 45.3799605),
        (6.652721, 45.3729675),
    ])
}

#[fixture]
fn simple_segment() -> TrackSegment {
    segment(&[
        (6.6534126, 45.3866878, None),
        (6.6532833, 45.386625, None),
        (6.6532399, 45.3865363, None),
        (6.6532228, 45.3862921, None),
        (6.6531947, 45.3859249, None),
        (6.6532034, 45.3855642, None),
        (6.6531879, 45.3851181, None),
        (6.6532009, 45.384639, None),
        (6.653174, 45.3841198, None),
        (6.6531506, 45.3836928, None),
        (6.6531397, 45.3831716, None),
        (6.6531375, 45.3826413, None),
        (6.6531095, 45.382125, None),
        (6.6530781, 45.3815575, None),
        (6.6530651, 45.3810034, None),
        (6.6530332, 45.3803867, None),
        (6.6530319, 45.3797959, None),
        (6.6530443, 45.3791876, None),
        (6.6529942, 45.3785015, None),
        (6.6529849, 45.3779231, None),
        (6.652974, 45.3773365, None),
        (6.6529653, 45.3767615, None),
        (6.6529501, 45.3759228, None),
        (6.6529276, 45.375154, None),
        (6.6529115, 45.3743625, None),
        (6.6528989, 45.3735875, None),
        (6.6528952, 45.3730052, None),
        (6.6528336, 45.3728979, None),
        (6.652673, 45.3728196, None),
        (6.6524959, 45.3727732, None),
    ])
}

#[fixture]
fn get_out_segment() -> TrackSegment {
    segment(&[
        (6.653481, 45.3867157, None),
        (6.653287, 45.386661, None),
        (6.6532186, 45.3864822, None),
        (6.6531804, 45.3860627, None),
        (6.6532004, 45.3857871, None),
        (6.6531734, 45.3855073, None),
        (6.6531621, 45.3851792, None),
        (6.6531452, 45.384639, None),
        (6.6531242, 45.3841532, None),
        (6.6531254, 45.3835527, None),
        (6.6531127, 45.3831058, None),
        (6.6530854, 45.3826733, None),
        (6.6530683, 45.3820477, None),
        (6.6530599, 45.3815762, None),
        (6.6530097, 45.3809965, None),
        (6.652996, 45.3805637, None),
        (6.6530062, 45.3800431, None),
        (6.6530017, 45.3795234, None),
        (6.652965, 45.3789977, None),
        (6.6529807, 45.3785376, None),
        (6.6529697, 45.3780693, None),
        (6.6529628, 45.3775987, None),
        (6.6529522, 45.3772738, None),
        (6.6529497, 45.3768569, None),
        (6.6529415, 45.3763606, None),
        (6.6529365, 45.3759643, None),
        (6.6529269, 45.3757525, None),
        (6.6529863, 45.3757313, None),
        (6.6531311, 45.375765, None),
        (6.6533365, 45.3758586, None),
    ])
}

#[rstest]
fn simple(_init: Init, line00: LineString, simple_segment: TrackSegment) {
    let s =
        ski_area(vec![lift("Lift 1".to_string(), line00, &[], false, false)]);
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(&g);

    let actual = find_lift_usage(&s, &segments);
    let expected: Vec<Activity> = vec![
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 0), (0, 2)),
        },
        Activity {
            type_: ActivityType::UseLift(UseLift {
                lift: &s.lifts[0],
                begin_time: None,
                end_time: None,
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            route: get_segment_part(&segments, (0, 2), (0, 28)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 28), (0, 30)),
        },
    ];
    assert!(
        ptrize_activities(&actual) == ptrize_activities(&expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

#[rstest]
fn simple_reverse_bad(
    _init: Init,
    line00: LineString,
    mut simple_segment: TrackSegment,
) {
    let s =
        ski_area(vec![lift("Lift 1".to_string(), line00, &[], false, false)]);
    simple_segment.points.reverse();
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(&g);

    let actual = find_lift_usage(&s, &segments);
    let expected: Vec<Activity> = vec![Activity {
        type_: ActivityType::Unknown,
        route: get_segment_part(&segments, (0, 0), (0, 30)),
    }];
    assert!(
        ptrize_activities(&actual) == ptrize_activities(&expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

#[rstest]
fn simple_reverse_good(
    _init: Init,
    line00: LineString,
    mut simple_segment: TrackSegment,
) {
    let s =
        ski_area(vec![lift("Lift 1".to_string(), line00, &[], true, false)]);
    simple_segment.points.reverse();
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(&g);

    let actual = find_lift_usage(&s, &segments);
    let expected: Vec<Activity> = vec![
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 0), (0, 2)),
        },
        Activity {
            type_: ActivityType::UseLift(UseLift {
                lift: &s.lifts[0],
                begin_time: None,
                end_time: None,
                begin_station: Some(1),
                end_station: Some(0),
                is_reverse: true,
            }),
            route: get_segment_part(&segments, (0, 2), (0, 28)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 28), (0, 30)),
        },
    ];
    assert!(
        ptrize_activities(&actual) == ptrize_activities(&expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

#[rstest]
fn get_out_bad(_init: Init, line00: LineString, get_out_segment: TrackSegment) {
    let s =
        ski_area(vec![lift("Lift 1".to_string(), line00, &[], false, false)]);
    let g = make_gpx(vec![get_out_segment]);
    let segments = get_segments(&g);

    let actual = find_lift_usage(&s, &segments);
    let expected: Vec<Activity> = vec![Activity {
        type_: ActivityType::Unknown,
        route: get_segment_part(&segments, (0, 0), (0, 30)),
    }];
    assert!(
        ptrize_activities(&actual) == ptrize_activities(&expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

#[rstest]
fn get_out_good(
    _init: Init,
    line00: LineString,
    get_out_segment: TrackSegment,
) {
    let s =
        ski_area(vec![lift("Lift 1".to_string(), line00, &[], false, true)]);
    let g = make_gpx(vec![get_out_segment]);
    let segments = get_segments(&g);

    let actual = find_lift_usage(&s, &segments);
    let expected: Vec<Activity> = vec![
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 0), (0, 2)),
        },
        Activity {
            type_: ActivityType::UseLift(UseLift {
                lift: &s.lifts[0],
                begin_time: None,
                end_time: None,
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            route: get_segment_part(&segments, (0, 2), (0, 28)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 28), (0, 30)),
        },
    ];
    assert!(
        ptrize_activities(&actual) == ptrize_activities(&expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}
