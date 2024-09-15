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

// https://www.openstreetmap.org/way/107110280
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

// https://www.openstreetmap.org/way/107110291
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

// https://www.openstreetmap.org/way/29409793
#[fixture]
fn line10() -> LineString {
    line(&[
        (6.651274, 45.3723031),
        (6.6499512, 45.3720526),
        (6.6489541, 45.3718638),
        (6.6471142, 45.3715154),
        (6.6467345, 45.3714435),
        (6.6451561, 45.371145),
    ])
}

#[fixture]
fn simple_segment() -> TrackSegment {
    segment(&[
        (6.6535288, 45.3867302, None),
        (6.6532869, 45.38669, None),
        (6.6531776, 45.3865113, None),
        (6.6531858, 45.3862492, None),
        (6.6531677, 45.3856584, None),
        (6.6531504, 45.3849873, None),
        (6.6530808, 45.3843306, None),
        (6.6530491, 45.383612, None),
        (6.6530706, 45.3830916, None),
        (6.6530409, 45.3821497, None),
        (6.6530034, 45.3811047, None),
        (6.6529511, 45.3799615, None),
        (6.6529481, 45.3789123, None),
        (6.652917, 45.3775815, None),
        (6.6528976, 45.3767035, None),
        (6.6528854, 45.3756355, None),
        (6.65286, 45.374517, None),
        (6.6528416, 45.373669, None),
        (6.652823, 45.3729593, None),
        (6.6526319, 45.372774, None),
        (6.6521592, 45.3726268, None),
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

#[fixture]
fn get_in_segment() -> TrackSegment {
    segment(&[
        (6.6534183, 45.3787864, None),
        (6.6534981, 45.3795387, None),
        (6.6535018, 45.3802503, None),
        (6.6529284, 45.3799891, None),
        (6.6529246, 45.3792279, None),
        (6.6528963, 45.3778658, None),
        (6.6528719, 45.3770813, None),
        (6.6528515, 45.3757549, None),
        (6.6528537, 45.3745336, None),
        (6.6528411, 45.3734302, None),
        (6.6528215, 45.3729541, None),
        (6.6524674, 45.3726633, None),
        (6.651844, 45.3725326, None),
        (6.6516495, 45.3726068, None),
    ])
}

#[fixture]
fn multiple_distinct_lifts_segment() -> TrackSegment {
    segment(&[
        (6.6535797, 45.3867922, None),
        (6.6533896, 45.3866573, None),
        (6.6532362, 45.3865374, None),
        (6.6531589, 45.3851786, None),
        (6.6531593, 45.3838609, None),
        (6.6530879, 45.3823978, None),
        (6.6529727, 45.3806109, None),
        (6.6529487, 45.3786397, None),
        (6.652905, 45.3765097, None),
        (6.6529288, 45.3745468, None),
        (6.6528969, 45.3736829, None),
        (6.6528515, 45.3729283, None),
        (6.6526247, 45.3727509, None),
        (6.6522609, 45.3726189, None),
        (6.6518038, 45.372672, None),
        (6.6513602, 45.3723656, None),
        (6.6506546, 45.3722414, None),
        (6.6493521, 45.3719711, None),
        (6.6479469, 45.3716735, None),
        (6.6463922, 45.3714093, None),
        (6.6452047, 45.3711732, None),
        (6.6445702, 45.3712323, None),
        (6.643845, 45.3714483, None),
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
            route: get_segment_part(&segments, (0, 2), (0, 19)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 19), (0, 21)),
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
        route: get_segment_part(&segments, (0, 0), (0, 21)),
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
            route: get_segment_part(&segments, (0, 2), (0, 19)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 19), (0, 21)),
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

#[rstest]
fn get_out_good_multisegment(
    _init: Init,
    line00: LineString,
    mut get_out_segment: TrackSegment,
) {
    let s =
        ski_area(vec![lift("Lift 1".to_string(), line00, &[], false, false)]);
    let mut seg2 = TrackSegment::new();
    seg2.points
        .append(&mut get_out_segment.points.split_off(28));
    let g = make_gpx(vec![get_out_segment, seg2]);
    let segments = get_segments(&g);
    eprintln!("{:#?}", g);

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
            route: get_segment_part(&segments, (1, 0), (1, 2)),
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
fn get_in_bad(_init: Init, line00: LineString, get_in_segment: TrackSegment) {
    let s =
        ski_area(vec![lift("Lift 1".to_string(), line00, &[], false, false)]);
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(&g);

    let actual = find_lift_usage(&s, &segments);
    let expected: Vec<Activity> = vec![Activity {
        type_: ActivityType::Unknown,
        route: get_segment_part(&segments, (0, 0), (0, 14)),
    }];
    assert!(
        ptrize_activities(&actual) == ptrize_activities(&expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}

#[rstest]
fn get_in_good_multisegment(
    _init: Init,
    line00: LineString,
    mut get_in_segment: TrackSegment,
) {
    let mut get_in_segment_2 = TrackSegment::new();
    get_in_segment_2
        .points
        .append(&mut get_in_segment.points.drain(3..).collect());
    let s =
        ski_area(vec![lift("Lift 1".to_string(), line00, &[], false, false)]);
    let g = make_gpx(vec![get_in_segment, get_in_segment_2]);
    let segments = get_segments(&g);

    let actual = find_lift_usage(&s, &segments);
    let expected: Vec<Activity> = vec![
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 0), (0, 3)),
        },
        Activity {
            type_: ActivityType::UseLift(UseLift {
                lift: &s.lifts[0],
                begin_time: None,
                end_time: None,
                begin_station: None,
                end_station: Some(1),
                is_reverse: false,
            }),
            route: get_segment_part(&segments, (1, 0), (1, 8)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (1, 8), (1, 11)),
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
fn multiple_distinct_lifts(
    _init: Init,
    line00: LineString,
    line10: LineString,
    multiple_distinct_lifts_segment: TrackSegment,
) {
    let s = ski_area(vec![
        lift("Lift 1".to_string(), line00, &[], false, false),
        lift("Lift 2".to_string(), line10, &[], false, false),
    ]);
    let g = make_gpx(vec![multiple_distinct_lifts_segment]);
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
            route: get_segment_part(&segments, (0, 2), (0, 12)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 12), (0, 15)),
        },
        Activity {
            type_: ActivityType::UseLift(UseLift {
                lift: &s.lifts[1],
                begin_time: None,
                end_time: None,
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            route: get_segment_part(&segments, (0, 15), (0, 21)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 21), (0, 23)),
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
fn multiple_lifts_same_start_take_longer(
    _init: Init,
    mut line00: LineString,
    line01: LineString,
    simple_segment: TrackSegment,
) {
    line00.0.pop();
    let s = ski_area(vec![
        lift("Lift 1".to_string(), line00, &[], false, false),
        lift("Lift 2".to_string(), line01, &[], false, false),
    ]);
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
                lift: &s.lifts[1],
                begin_time: None,
                end_time: None,
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            route: get_segment_part(&segments, (0, 2), (0, 19)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 19), (0, 21)),
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
fn multiple_lifts_same_end_take_longer(
    _init: Init,
    line00: LineString,
    mut line01: LineString,
    simple_segment: TrackSegment,
) {
    line01.0.remove(0);
    let s = ski_area(vec![
        lift("Lift 1".to_string(), line00, &[], false, false),
        lift("Lift 2".to_string(), line01, &[], false, false),
    ]);
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
            route: get_segment_part(&segments, (0, 2), (0, 19)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 19), (0, 21)),
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
fn multiple_lifts_same_end_take_shorter(
    _init: Init,
    mut line00: LineString,
    line01: LineString,
    get_in_segment: TrackSegment,
) {
    line00.0.drain(0..3);
    let s = ski_area(vec![
        lift("Lift 1".to_string(), line00, &[], false, false),
        lift("Lift 2".to_string(), line01, &[], false, false),
    ]);
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(&g);

    let actual = find_lift_usage(&s, &segments);
    let expected: Vec<Activity> = vec![
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 0), (0, 3)),
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
            route: get_segment_part(&segments, (0, 3), (0, 11)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 11), (0, 14)),
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
fn multiple_lifts_same_start_take_shorter(
    _init: Init,
    mut line00: LineString,
    mut line01: LineString,
    mut get_in_segment: TrackSegment,
) {
    line00.0.reverse();
    line01.0.reverse();
    get_in_segment.points.reverse();
    line01.0.truncate(2);
    let s = ski_area(vec![
        lift("Lift 1".to_string(), line00, &[], false, false),
        lift("Lift 2".to_string(), line01, &[], false, false),
    ]);
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(&g);

    let actual = find_lift_usage(&s, &segments);
    let expected: Vec<Activity> = vec![
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 0), (0, 3)),
        },
        Activity {
            type_: ActivityType::UseLift(UseLift {
                lift: &s.lifts[1],
                begin_time: None,
                end_time: None,
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            route: get_segment_part(&segments, (0, 3), (0, 11)),
        },
        Activity {
            type_: ActivityType::Unknown,
            route: get_segment_part(&segments, (0, 11), (0, 14)),
        },
    ];
    assert!(
        ptrize_activities(&actual) == ptrize_activities(&expected),
        "Actual: {:#?}\nExpected: {:#?}",
        actual,
        expected
    );
}
