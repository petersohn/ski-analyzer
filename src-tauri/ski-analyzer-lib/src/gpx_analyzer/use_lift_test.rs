use super::test_util::save_analyzed_route;
use super::use_lift::find_lift_usage;
use super::Segments;
use super::{Activity, ActivityType, LiftEnd, UseLift};
use crate::assert_eq_pretty;
use crate::ski_area::{Lift, PointWithElevation, SkiArea};
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::rect::union_rects_all;
use crate::utils::test_util::{init, save_ski_area, Init};

use ::function_name::named;
use geo::{coord, point, LineString, Rect};
use gpx::{Gpx, Track, TrackSegment, Waypoint};
use rstest::{fixture, rstest};
use std::collections::HashMap;
use std::fs;
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
    let line = BoundedGeometry::new(line_).unwrap();
    Lift {
        ref_: String::new(),
        name,
        type_: String::new(),
        line,
        stations,
        lengths: Vec::new(),
        can_go_reverse,
        can_disembark,
    }
}

fn ski_area(name: &str, lifts: Vec<Lift>) -> SkiArea {
    SkiArea::new(
        name.to_string(),
        lifts.into_iter().map(|l| (l.name.clone(), l)).collect(),
        HashMap::new(),
        OffsetDateTime::UNIX_EPOCH,
    )
    .unwrap()
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

fn get_segments(gpx: Gpx) -> Segments {
    gpx.tracks
        .into_iter()
        .map(|t| t.segments.into_iter().map(|s| s.points))
        .flatten()
        .collect()
}

#[derive(PartialEq, Eq)]
pub struct UseLiftPtr {
    lift: *const Lift,
    begin_station: LiftEnd,
    end_station: LiftEnd,
    is_reverse: bool,
}

fn get_segment_part(
    segments: &Segments,
    begin: (usize, usize),
    end: (usize, usize),
) -> Segments {
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
fn get_out_segment_2() -> TrackSegment {
    segment(&[
        (6.6538573, 45.3870086, None),
        (6.6535056, 45.3867172, None),
        (6.6531744, 45.3865044, None),
        (6.6531698, 45.3857799, None),
        (6.6531392, 45.3847045, None),
        (6.65308, 45.3835778, None),
        (6.6530592, 45.382487, None),
        (6.6530045, 45.3813324, None),
        (6.6530927, 45.3817535, None),
        (6.6535936, 45.3819592, None),
        (6.6540583, 45.3827142, None),
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

#[fixture]
fn zigzag_segment() -> TrackSegment {
    segment(&[
        (6.6535194, 45.3867225, None),
        (6.653277, 45.3866553, None),
        (6.6531967, 45.3864826, None),
        (6.6531896, 45.3857757, None),
        (6.653133, 45.3847939, None),
        (6.6530929, 45.3835549, None),
        (6.6530434, 45.3823099, None),
        (6.6529928, 45.3809593, None),
        (6.652991, 45.3806471, None),
        (6.6529965, 45.3805166, None),
        (6.6529725, 45.3805184, None),
        (6.6530072, 45.3805177, None),
        (6.6529881, 45.3805277, None),
        (6.6529916, 45.3805075, None),
        (6.6530022, 45.3805263, None),
        (6.6529789, 45.3805119, None),
        (6.653002, 45.3805101, None),
        (6.652995, 45.3804726, None),
        (6.6529893, 45.3803492, None),
        (6.6529571, 45.379545, None),
        (6.6529393, 45.3785823, None),
        (6.6529232, 45.3776578, None),
        (6.6529226, 45.3767989, None),
        (6.6528916, 45.3757027, None),
        (6.6528758, 45.3745658, None),
        (6.652851, 45.3736014, None),
        (6.6528168, 45.3729555, None),
        (6.6526163, 45.372802, None),
        (6.6522554, 45.3726394, None),
    ])
}

#[fixture]
fn restart_segment_1() -> TrackSegment {
    segment(&[
        (6.6535209, 45.3867218, None),
        (6.6533266, 45.3866286, None),
        (6.6531925, 45.3865133, None),
        (6.6531977, 45.3862565, None),
        (6.6531653, 45.3856013, None),
        (6.6531452, 45.3848611, None),
        (6.653079, 45.3834045, None),
        (6.6529836, 45.3814095, None),
        (6.6529546, 45.3800842, None),
    ])
}

#[fixture]
fn restart_segment_2() -> TrackSegment {
    segment(&[
        (6.6530334, 45.3830165, None),
        (6.653021, 45.3814001, None),
        (6.652934, 45.3793176, None),
        (6.6529175, 45.3774651, None),
        (6.6528885, 45.3754106, None),
        (6.6528969, 45.3737299, None),
        (6.6527973, 45.3729563, None),
        (6.6526026, 45.3727424, None),
        (6.6519628, 45.3725725, None),
    ])
}

fn run(s: &SkiArea, segments: Segments, expected: Vec<Activity>, name: &str) {
    let dir = format!("test_output/use_lift_test/{}", name);
    fs::create_dir_all(&dir).unwrap();
    save_ski_area(s, &format!("{}/ski_area.json", dir));

    let bounding_rect = union_rects_all(
        segments
            .iter()
            .flatten()
            .map(|wp| Rect::new(wp.point(), wp.point())),
    )
    .unwrap();
    let expected_route = BoundedGeometry {
        item: expected,
        bounding_rect,
    };
    save_analyzed_route(&expected_route, &format!("{}/expected.json", dir));

    let actual = find_lift_usage(s, segments);
    let actual_route = BoundedGeometry {
        item: actual,
        bounding_rect,
    };
    save_analyzed_route(&actual_route, &format!("{}/actual.json", dir));

    assert_eq_pretty!(actual_route.item, expected_route.item);
}

#[rstest]
#[named]
fn simple(_init: Init, line00: LineString, simple_segment: TrackSegment) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(g);
    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 19), (0, 21)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn simple_reverse_bad(
    _init: Init,
    line00: LineString,
    mut simple_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    simple_segment.points.reverse();
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![Activity::new(
        ActivityType::Unknown(()),
        get_segment_part(&segments, (0, 0), (0, 21)),
    )];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn simple_reverse_good(
    _init: Init,
    line00: LineString,
    mut simple_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], true, false)],
    );
    simple_segment.points.reverse();
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(1),
                end_station: Some(0),
                is_reverse: true,
            }),
            get_segment_part(&segments, (0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 19), (0, 21)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn get_out_bad(_init: Init, line00: LineString, get_out_segment: TrackSegment) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    let g = make_gpx(vec![get_out_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![Activity::new(
        ActivityType::Unknown(()),
        get_segment_part(&segments, (0, 0), (0, 30)),
    )];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn get_out_good(
    _init: Init,
    line00: LineString,
    get_out_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, true)],
    );
    let g = make_gpx(vec![get_out_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 28)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 28), (0, 30)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn get_out_good_2(
    _init: Init,
    line00: LineString,
    get_out_segment_2: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, true)],
    );
    let g = make_gpx(vec![get_out_segment_2]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 8)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 8), (0, 11)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn get_out_good_multisegment(
    _init: Init,
    line00: LineString,
    mut get_out_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    let mut seg2 = TrackSegment::new();
    seg2.points
        .append(&mut get_out_segment.points.split_off(28));
    let g = make_gpx(vec![get_out_segment, seg2]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 28)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (1, 0), (1, 2)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn get_in_bad(_init: Init, line00: LineString, get_in_segment: TrackSegment) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![Activity::new(
        ActivityType::Unknown(()),
        get_segment_part(&segments, (0, 0), (0, 14)),
    )];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn get_in_good_multisegment(
    _init: Init,
    line00: LineString,
    mut get_in_segment: TrackSegment,
) {
    let mut get_in_segment_2 = TrackSegment::new();
    get_in_segment_2
        .points
        .append(&mut get_in_segment.points.drain(3..).collect());
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    let g = make_gpx(vec![get_in_segment, get_in_segment_2]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: None,
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (1, 0), (1, 8)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (1, 8), (1, 11)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn multiple_distinct_lifts(
    _init: Init,
    line00: LineString,
    line10: LineString,
    multiple_distinct_lifts_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![
            lift("Lift 1".to_string(), line00, &[], false, false),
            lift("Lift 2".to_string(), line10, &[], false, false),
        ],
    );
    let g = make_gpx(vec![multiple_distinct_lifts_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 12)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 12), (0, 15)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 2".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 15), (0, 21)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 21), (0, 23)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn multiple_lifts_same_start_take_longer(
    _init: Init,
    mut line00: LineString,
    line01: LineString,
    simple_segment: TrackSegment,
) {
    line00.0.pop();
    let s = ski_area(
        function_name!(),
        vec![
            lift("Lift 1".to_string(), line00, &[], false, false),
            lift("Lift 2".to_string(), line01, &[], false, false),
        ],
    );
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 2".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 19), (0, 21)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn multiple_lifts_same_end_take_longer(
    _init: Init,
    line00: LineString,
    mut line01: LineString,
    simple_segment: TrackSegment,
) {
    line01.0.remove(0);
    let s = ski_area(
        function_name!(),
        vec![
            lift("Lift 1".to_string(), line00, &[], false, false),
            lift("Lift 2".to_string(), line01, &[], false, false),
        ],
    );
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 19), (0, 21)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn multiple_lifts_same_end_take_shorter(
    _init: Init,
    mut line00: LineString,
    line01: LineString,
    get_in_segment: TrackSegment,
) {
    line00.0.drain(0..3);
    let s = ski_area(
        function_name!(),
        vec![
            lift("Lift 1".to_string(), line00, &[], false, false),
            lift("Lift 2".to_string(), line01, &[], false, false),
        ],
    );
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 3), (0, 11)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 11), (0, 14)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
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
    let s = ski_area(
        function_name!(),
        vec![
            lift("Lift 1".to_string(), line00, &[], false, false),
            lift("Lift 2".to_string(), line01, &[], false, false),
        ],
    );
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 2".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 3), (0, 11)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 11), (0, 14)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn midstation_get_in(
    _init: Init,
    line00: LineString,
    get_in_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[3], false, false)],
    );
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(1),
                end_station: Some(2),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 3), (0, 11)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 11), (0, 14)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn midstation_get_out(
    _init: Init,
    mut line00: LineString,
    mut get_in_segment: TrackSegment,
) {
    line00.0.reverse();
    get_in_segment.points.reverse();
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[1], false, false)],
    );
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 3), (0, 11)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 11), (0, 14)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn parallel_lifts(
    _init: Init,
    line00: LineString,
    line01: LineString,
    simple_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![
            lift("Lift 1".to_string(), line00, &[], false, false),
            lift("Lift 2".to_string(), line01, &[], false, false),
        ],
    );
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 19), (0, 21)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn zigzag(_init: Init, line00: LineString, zigzag_segment: TrackSegment) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    let g = make_gpx(vec![zigzag_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 27)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 27), (0, 29)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn restart_with_new_segment(
    _init: Init,
    line00: LineString,
    restart_segment_1: TrackSegment,
    restart_segment_2: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    let g = make_gpx(vec![restart_segment_1, restart_segment_2]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (0, 0), (0, 2)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            get_segment_part(&segments, (0, 2), (0, 9)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: None,
                end_station: Some(1),
                is_reverse: false,
            }),
            get_segment_part(&segments, (1, 0), (1, 7)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            get_segment_part(&segments, (1, 7), (1, 9)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}
