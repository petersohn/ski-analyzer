use super::test_util::save_analyzed_route;
use super::use_lift::find_lift_usage;
use super::Segments;
use super::{Activity, ActivityType, LiftEnd, UseLift};
use crate::assert_eq_pretty;
use crate::ski_area::{Lift, PointWithElevation, SkiArea};
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::cancel::CancellationToken;
use crate::utils::rect::union_rects_all;
use crate::utils::test_util::{
    create_ski_area_metadata, get_segments, init, make_gpx, save_ski_area,
    segment, Init,
};

use ::function_name::named;
use geo::{coord, Distance, Haversine, LineString, Rect};
use gpx::TrackSegment;
use rstest::{fixture, rstest};
use std::collections::HashMap;
use std::fs;
use time::{Duration, OffsetDateTime};

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
        create_ski_area_metadata(name.to_string()),
        lifts.into_iter().map(|l| (l.name.clone(), l)).collect(),
        HashMap::new(),
        OffsetDateTime::UNIX_EPOCH,
    )
    .unwrap()
}

#[derive(PartialEq, Eq)]
pub struct UseLiftPtr {
    lift: *const Lift,
    begin_station: LiftEnd,
    end_station: LiftEnd,
    is_reverse: bool,
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
        (6.6535288, 45.3867302),
        (6.6532869, 45.38669),
        (6.6531776, 45.3865113),
        (6.6531858, 45.3862492),
        (6.6531677, 45.3856584),
        (6.6531504, 45.3849873),
        (6.6530808, 45.3843306),
        (6.6530491, 45.383612),
        (6.6530706, 45.3830916),
        (6.6530409, 45.3821497),
        (6.6530034, 45.3811047),
        (6.6529511, 45.3799615),
        (6.6529481, 45.3789123),
        (6.652917, 45.3775815),
        (6.6528976, 45.3767035),
        (6.6528854, 45.3756355),
        (6.65286, 45.374517),
        (6.6528416, 45.373669),
        (6.652823, 45.3729593),
        (6.6526319, 45.372774),
        (6.6521592, 45.3726268),
    ])
}

#[fixture]
fn get_out_segment() -> TrackSegment {
    segment(&[
        (6.653481, 45.3867157),
        (6.653287, 45.386661),
        (6.6532186, 45.3864822),
        (6.6531804, 45.3860627),
        (6.6532004, 45.3857871),
        (6.6531734, 45.3855073),
        (6.6531621, 45.3851792),
        (6.6531452, 45.384639),
        (6.6531242, 45.3841532),
        (6.6531254, 45.3835527),
        (6.6531127, 45.3831058),
        (6.6530854, 45.3826733),
        (6.6530683, 45.3820477),
        (6.6530599, 45.3815762),
        (6.6530097, 45.3809965),
        (6.652996, 45.3805637),
        (6.6530062, 45.3800431),
        (6.6530017, 45.3795234),
        (6.652965, 45.3789977),
        (6.6529807, 45.3785376),
        (6.6529697, 45.3780693),
        (6.6529628, 45.3775987),
        (6.6529522, 45.3772738),
        (6.6529497, 45.3768569),
        (6.6529415, 45.3763606),
        (6.6529365, 45.3759643),
        (6.6529269, 45.3757525),
        (6.6529863, 45.3757313),
        (6.6531311, 45.375765),
        (6.6533365, 45.3758586),
    ])
}

#[fixture]
fn get_out_segment_2() -> TrackSegment {
    segment(&[
        (6.6538573, 45.3870086),
        (6.6535056, 45.3867172),
        (6.6531744, 45.3865044),
        (6.6531698, 45.3857799),
        (6.6531392, 45.3847045),
        (6.65308, 45.3835778),
        (6.6530592, 45.382487),
        (6.6530045, 45.3813324),
        (6.6530927, 45.3817535),
        (6.6535936, 45.3819592),
        (6.6540583, 45.3827142),
    ])
}

#[fixture]
fn get_in_segment() -> TrackSegment {
    segment(&[
        (6.6534183, 45.3787864),
        (6.6534981, 45.3795387),
        (6.6535018, 45.3802503),
        (6.6529284, 45.3799891),
        (6.6529246, 45.3792279),
        (6.6528963, 45.3778658),
        (6.6528719, 45.3770813),
        (6.6528515, 45.3757549),
        (6.6528537, 45.3745336),
        (6.6528411, 45.3734302),
        (6.6528215, 45.3729541),
        (6.6524674, 45.3726633),
        (6.651844, 45.3725326),
        (6.6516495, 45.3726068),
    ])
}

#[fixture]
fn multiple_distinct_lifts_segment() -> TrackSegment {
    segment(&[
        (6.6535797, 45.3867922),
        (6.6533896, 45.3866573),
        (6.6532362, 45.3865374),
        (6.6531589, 45.3851786),
        (6.6531593, 45.3838609),
        (6.6530879, 45.3823978),
        (6.6529727, 45.3806109),
        (6.6529487, 45.3786397),
        (6.652905, 45.3765097),
        (6.6529288, 45.3745468),
        (6.6528969, 45.3736829),
        (6.6528515, 45.3729283),
        (6.6526247, 45.3727509),
        (6.6522609, 45.3726189),
        (6.6518038, 45.372672),
        (6.6513602, 45.3723656),
        (6.6506546, 45.3722414),
        (6.6493521, 45.3719711),
        (6.6479469, 45.3716735),
        (6.6463922, 45.3714093),
        (6.6452047, 45.3711732),
        (6.6445702, 45.3712323),
        (6.643845, 45.3714483),
    ])
}

#[fixture]
fn zigzag_segment() -> TrackSegment {
    segment(&[
        (6.6535194, 45.3867225),
        (6.653277, 45.3866553),
        (6.6531967, 45.3864826),
        (6.6531896, 45.3857757),
        (6.653133, 45.3847939),
        (6.6530929, 45.3835549),
        (6.6530434, 45.3823099),
        (6.6529928, 45.3809593),
        (6.652991, 45.3806471),
        (6.6529965, 45.3805166),
        (6.6529725, 45.3805184),
        (6.6530072, 45.3805177),
        (6.6529881, 45.3805277),
        (6.6529916, 45.3805075),
        (6.6530022, 45.3805263),
        (6.6529789, 45.3805119),
        (6.653002, 45.3805101),
        (6.652995, 45.3804726),
        (6.6529893, 45.3803492),
        (6.6529571, 45.379545),
        (6.6529393, 45.3785823),
        (6.6529232, 45.3776578),
        (6.6529226, 45.3767989),
        (6.6528916, 45.3757027),
        (6.6528758, 45.3745658),
        (6.652851, 45.3736014),
        (6.6528168, 45.3729555),
        (6.6526163, 45.372802),
        (6.6522554, 45.3726394),
    ])
}

#[fixture]
fn restart_segment_1() -> TrackSegment {
    segment(&[
        (6.6535209, 45.3867218),
        (6.6533266, 45.3866286),
        (6.6531925, 45.3865133),
        (6.6531977, 45.3862565),
        (6.6531653, 45.3856013),
        (6.6531452, 45.3848611),
        (6.653079, 45.3834045),
        (6.6529836, 45.3814095),
        (6.6529546, 45.3800842),
    ])
}

#[fixture]
fn restart_segment_2() -> TrackSegment {
    segment(&[
        (6.6530334, 45.3830165),
        (6.653021, 45.3814001),
        (6.652934, 45.3793176),
        (6.6529175, 45.3774651),
        (6.6528885, 45.3754106),
        (6.6528969, 45.3737299),
        (6.6527973, 45.3729563),
        (6.6526026, 45.3727424),
        (6.6519628, 45.3725725),
    ])
}

#[fixture]
fn speed_segment() -> TrackSegment {
    segment(&[
        (6.6535665, 45.3866982),
        (6.6533672, 45.3866708),
        (6.653225, 45.3866106),
        (6.6532195, 45.3865668),
        (6.6532195, 45.3865115),
        (6.6532143, 45.3864621),
        (6.6532081, 45.3864061),
        (6.6532091, 45.3863586),
        (6.6531953, 45.3860718),
        (6.6531088, 45.3831123),
        (6.6530512, 45.3805724),
        (6.6529744, 45.3771722),
        (6.6529023, 45.3741731),
        (6.6528822, 45.3730672),
        (6.6528771, 45.3730199),
        (6.6528678, 45.3729821),
        (6.652844, 45.372949),
        (6.6528103, 45.3729172),
        (6.6527781, 45.3728786),
        (6.652594, 45.3728038),
        (6.6524039, 45.3728215),
    ])
}

fn add_speed(segment: &mut TrackSegment, speeds: &[(f64, usize)]) {
    let mut time = OffsetDateTime::UNIX_EPOCH;
    segment.points[0].time = Some(time.clone().into());
    let mut it = speeds.iter();
    let mut current = it.next().unwrap();
    let mut count = 0;
    let ps = &mut segment.points;
    for i in 1..ps.len() {
        let distance = Haversine::distance(ps[i - 1].point(), ps[i].point());
        time += Duration::seconds_f64(distance / current.0);
        ps[i].time = Some(time.clone().into());
        if current.1 > 0 {
            count += 1;
            if count == current.1 {
                current = it.next().unwrap();
                count = 0;
            }
        }
    }
}

fn run(s: &SkiArea, segments: Segments, expected: Vec<Activity>, name: &str) {
    let dir = format!("test_output/use_lift_test/{}", name);
    fs::create_dir_all(&dir).unwrap();
    save_ski_area(s, &format!("{}/ski_area.json", dir));

    let bounding_rect = union_rects_all(
        segments
            .0
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

    let actual =
        find_lift_usage(&CancellationToken::new(), s, segments).unwrap();
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
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 18), (0, 21)),
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
        segments.clone_part((0, 0), (0, 21)),
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
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(1),
                end_station: Some(0),
                is_reverse: true,
            }),
            segments.clone_part((0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 18), (0, 21)),
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
        segments.clone_part((0, 0), (0, 30)),
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
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 28)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 27), (0, 30)),
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
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 8)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 7), (0, 11)),
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
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (1, 0)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((1, 0), (1, 2)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn get_out_good_multisegment_multiple_candidates(
    _init: Init,
    line00: LineString,
    line01: LineString,
    mut get_out_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![
            lift("Lift 1".to_string(), line00, &[], false, false),
            lift("Lift 2".to_string(), line01, &[], false, false),
        ],
    );
    let mut seg2 = TrackSegment::new();
    seg2.points
        .append(&mut get_out_segment.points.split_off(28));
    get_out_segment.points.pop();
    let g = make_gpx(vec![get_out_segment, seg2]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (1, 0)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((1, 0), (1, 2)),
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
        segments.clone_part((0, 0), (0, 14)),
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
            segments.clone_part((0, 0), (1, 0)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: None,
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((1, 0), (1, 8)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((1, 7), (1, 11)),
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
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 12)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 11), (0, 16)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 2".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 15), (0, 21)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 20), (0, 23)),
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
            lift("Lift 1".to_string(), line00, &[], false, true),
            lift("Lift 2".to_string(), line01, &[], false, true),
        ],
    );
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 2".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 18), (0, 21)),
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
            lift("Lift 1".to_string(), line00, &[], false, true),
            lift("Lift 2".to_string(), line01, &[], false, true),
        ],
    );
    let g = make_gpx(vec![simple_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 18), (0, 21)),
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
            lift("Lift 1".to_string(), line00, &[], false, true),
            lift("Lift 2".to_string(), line01, &[], false, true),
        ],
    );
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 0), (0, 4)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 3), (0, 11)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 10), (0, 14)),
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
            lift("Lift 1".to_string(), line00, &[], false, true),
            lift("Lift 2".to_string(), line01, &[], false, true),
        ],
    );
    let g = make_gpx(vec![get_in_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 0), (0, 4)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 2".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 3), (0, 11)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 10), (0, 14)),
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
            segments.clone_part((0, 0), (0, 4)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(1),
                end_station: Some(2),
                is_reverse: false,
            }),
            segments.clone_part((0, 3), (0, 11)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 10), (0, 14)),
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
            segments.clone_part((0, 0), (0, 4)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 3), (0, 11)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 10), (0, 14)),
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
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 18), (0, 21)),
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
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 27)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 26), (0, 29)),
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
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: None,
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 9)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: None,
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((1, 0), (1, 7)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((1, 6), (1, 9)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn speed_always_fast(
    _init: Init,
    line00: LineString,
    mut speed_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    add_speed(&mut speed_segment, &[(2.0, 0)]);
    let g = make_gpx(vec![speed_segment]);
    let segments = get_segments(g);

    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: "Lift 1".to_string(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 2), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 18), (1, 0)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn speed_always_slow(
    _init: Init,
    line00: LineString,
    mut speed_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    add_speed(&mut speed_segment, &[(0.5, 0)]);
    let g = make_gpx(vec![speed_segment]);
    let segments = get_segments(g);

    let lift_id = "Lift 1".to_string();
    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::EnterLift(lift_id.clone()),
            segments.clone_part((0, 2), (0, 7)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: lift_id.clone(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 6), (0, 14)),
        ),
        Activity::new(
            ActivityType::ExitLift(lift_id.clone()),
            segments.clone_part((0, 13), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 18), (1, 0)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}

#[rstest]
#[named]
fn speed_slow_then_fast(
    _init: Init,
    line00: LineString,
    mut speed_segment: TrackSegment,
) {
    let s = ski_area(
        function_name!(),
        vec![lift("Lift 1".to_string(), line00, &[], false, false)],
    );
    add_speed(&mut speed_segment, &[(0.5, 4), (2.0, 11), (0.5, 0)]);
    let g = make_gpx(vec![speed_segment]);
    let segments = get_segments(g);

    let lift_id = "Lift 1".to_string();
    let expected: Vec<Activity> = vec![
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 0), (0, 3)),
        ),
        Activity::new(
            ActivityType::EnterLift(lift_id.clone()),
            segments.clone_part((0, 2), (0, 5)),
        ),
        Activity::new(
            ActivityType::UseLift(UseLift {
                lift_id: lift_id.clone(),
                begin_station: Some(0),
                end_station: Some(1),
                is_reverse: false,
            }),
            segments.clone_part((0, 4), (0, 16)),
        ),
        Activity::new(
            ActivityType::ExitLift(lift_id.clone()),
            segments.clone_part((0, 15), (0, 19)),
        ),
        Activity::new(
            ActivityType::Unknown(()),
            segments.clone_part((0, 18), (1, 0)),
        ),
    ];

    run(&s, segments, expected, function_name!());
}
