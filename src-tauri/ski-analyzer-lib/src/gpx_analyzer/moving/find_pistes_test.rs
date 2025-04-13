use ::function_name::named;
use geo::LineString;
use gpx::TrackSegment;
use std::collections::HashMap;
use std::fs;
use time::OffsetDateTime;

use super::find_pistes::find_pistes;
use super::{commit_moves, MoveType, Moving};
use crate::assert_eq_pretty;
use crate::gpx_analyzer::test_util::save_analyzed_route;
use crate::gpx_analyzer::{SegmentCoordinate, Segments};
use crate::ski_area::{Piste, SkiArea};
use crate::utils::cancel::CancellationToken;
use crate::utils::test_util::{
    create_ski_area_metadata, get_segments, make_gpx, piste, save_ski_area,
    segment,
};
use piste_data::{area0, area1_0, area1_1, area1_2, area1_3, line0, line1};

mod piste_data;

fn ski_area(name: &str, pistes: &[Piste]) -> SkiArea {
    let piste_map = pistes
        .iter()
        .map(|p| (p.metadata.name.clone(), p.clone()))
        .collect();

    let mut result = SkiArea::new(
        create_ski_area_metadata(name.to_string()),
        HashMap::new(),
        piste_map,
        OffsetDateTime::now_utc(),
    )
    .unwrap();
    result.clip_piste_lines();
    result
}

fn save_activities(
    path: &str,
    segments: &Segments,
    data: &Vec<(Moving, SegmentCoordinate)>,
) {
    let result = commit_moves(&mut segments.clone(), data.clone());
    save_analyzed_route(&result, path);
}

fn run_test(
    name: &str,
    ski_area: SkiArea,
    track_segments: Vec<TrackSegment>,
    input: Vec<(MoveType, SegmentCoordinate)>,
    expected: Vec<(Moving, SegmentCoordinate)>,
) {
    let dir = format!("test_output/find_pistes_test/{name}");
    fs::create_dir_all(&dir).unwrap();
    save_ski_area(&ski_area, &format!("{dir}/ski_area.json"));

    let gpx = make_gpx(track_segments);
    let segments = get_segments(gpx);
    save_activities(&format!("{dir}/expected.json"), &segments, &expected);

    let actual =
        find_pistes(&CancellationToken::new(), &ski_area, &segments, input)
            .unwrap();
    save_activities(&format!("{dir}/actual.json"), &segments, &actual);
    assert_eq_pretty!(actual, expected);
}

fn segments0() -> Vec<TrackSegment> {
    vec![segment(&[
        (6.5239132, 45.3109591),
        (6.5242407, 45.311227),
        (6.524431, 45.3117135),
        (6.5249571, 45.3120028),
        (6.5255282, 45.3122029),
        (6.5260645, 45.3123324),
        (6.5263981, 45.3127097),
        (6.5267808, 45.3130898),
    ])]
}

#[test]
#[named]
fn move_on_piste_with_line() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![line0()], vec![])]);
    let segments = segments0();
    let expected = vec![(
        Moving {
            piste_id: "1".to_string(),
            move_type: MoveType::Ski,
        },
        (0, 0),
    )];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn move_on_piste_with_area() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![], vec![area0()])]);
    let segments = segments0();
    let expected = vec![(
        Moving {
            piste_id: "1".to_string(),
            move_type: MoveType::Ski,
        },
        (0, 0),
    )];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

fn segments_enter_leave() -> Vec<TrackSegment> {
    vec![segment(&[
        (6.5233781, 45.3113157),
        (6.5235761, 45.311424),
        (6.5237633, 45.3115298),
        (6.5239088, 45.3116112),
        (6.5240496, 45.3117135),
        (6.5243262, 45.3117365),
        (6.5244469, 45.3118339),
        (6.5246273, 45.3119132),
        (6.5247572, 45.3119803),
        (6.5249056, 45.3120383),
        (6.5250872, 45.312071),
        (6.5252607, 45.3120567),
        (6.5254726, 45.3121102),
        (6.5256625, 45.3121874),
        (6.5258554, 45.3122888),
        (6.5261287, 45.3123904),
        (6.5263775, 45.3125148),
        (6.5264648, 45.3125818),
        (6.5265656, 45.3126546),
        (6.5267541, 45.3128186),
        (6.5268557, 45.3129231),
        (6.5269511, 45.3130399),
        (6.5270301, 45.3131337),
        (6.5271117, 45.3131904),
        (6.5272457, 45.3132652),
        (6.5273506, 45.3133451),
        (6.5275662, 45.3134669),
        (6.5277001, 45.3134394),
        (6.5279491, 45.3133244),
        (6.5284426, 45.3131153),
        (6.5287251, 45.3128361),
        (6.5288191, 45.3130131),
        (6.5285631, 45.3133886),
        (6.528277, 45.3137732),
        (6.5282089, 45.3139223),
        (6.5280927, 45.3141359),
        (6.5281566, 45.3142624),
        (6.5282446, 45.3144319),
        (6.5283465, 45.3145908),
        (6.5284681, 45.3147511),
        (6.5285968, 45.3148795),
        (6.528711, 45.3150084),
        (6.5288402, 45.3151749),
        (6.528908, 45.3153249),
        (6.5290303, 45.3154487),
    ])]
}

#[test]
#[named]
fn enter_and_leave_piste_with_area() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![], vec![area0()])]);
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 27),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 35),
        ),
    ];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn enter_and_leave_piste_with_line() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![line0()], vec![])]);
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 27),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 35),
        ),
    ];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn enter_and_leave_another_piste_with_area() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![], vec![area0()]),
            piste("2", vec![], vec![area1_3()]),
        ],
    );
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 27),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 35),
        ),
    ];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn enter_and_leave_another_piste_with_line() {
    let line1_part =
        LineString::new(line1().0.split_at(32).1.split_at(12).0.to_vec());
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![line0()], vec![]),
            piste("2", vec![line1_part], vec![]),
        ],
    );
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 26),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 36),
        ),
    ];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn enter_and_leave_another_piste_from_area_to_line() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![], vec![area0()]),
            piste("2", vec![line1()], vec![]),
        ],
    );
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 27),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 35),
        ),
    ];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn enter_and_leave_another_piste_from_line_to_area() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![line0()], vec![area0()]),
            piste("2", vec![], vec![area1_3()]),
        ],
    );
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 27),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 35),
        ),
    ];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

fn segments_follow_piste() -> Vec<TrackSegment> {
    vec![segment(&[
        (6.5266628, 45.3111721),
        (6.5265875, 45.3112152),
        (6.5264663, 45.3112827),
        (6.5263322, 45.3113849),
        (6.5261553, 45.311451),
        (6.5260355, 45.3115289),
        (6.5259002, 45.3116144),
        (6.5258323, 45.3117159),
        (6.5258779, 45.3117925),
        (6.525895, 45.3119079),
        (6.5259644, 45.3120047),
        (6.5260757, 45.3120565),
        (6.5261671, 45.3121841),
        (6.5262546, 45.3122757),
        (6.5262868, 45.3123771),
        (6.5262323, 45.3124737),
        (6.5262321, 45.3125694),
        (6.5261976, 45.3126598),
        (6.5261513, 45.3127399),
        (6.5261892, 45.3128161),
        (6.5262946, 45.3128463),
        (6.5264407, 45.3128072),
        (6.5266083, 45.3127555),
        (6.5267407, 45.3126876),
        (6.5269191, 45.3125976),
        (6.5270368, 45.3125484),
        (6.5271786, 45.3125683),
        (6.5272762, 45.312649),
        (6.5273379, 45.312706),
        (6.5273276, 45.3128047),
        (6.5273568, 45.3128811),
        (6.527279, 45.312994),
        (6.5272493, 45.3130743),
        (6.5271442, 45.3131802),
        (6.5271193, 45.3132978),
        (6.5271856, 45.3134415),
        (6.5273727, 45.313478),
        (6.5275434, 45.3134666),
        (6.5277459, 45.3134198),
        (6.5278877, 45.3133435),
        (6.5280262, 45.3132836),
        (6.5281814, 45.313218),
        (6.5283712, 45.3131632),
        (6.5285336, 45.3130777),
        (6.52862, 45.3129813),
        (6.5287133, 45.3128632),
        (6.5288536, 45.3128633),
        (6.5288654, 45.3129682),
        (6.5287551, 45.3130775),
        (6.5286816, 45.3132029),
        (6.5286133, 45.3133403),
        (6.528506, 45.3134612),
        (6.5283879, 45.3136126),
        (6.5282865, 45.3137681),
        (6.5282196, 45.3138874),
        (6.5281253, 45.314009),
    ])]
}

#[test]
#[named]
fn stay_on_piste_when_crossing_another_with_areas() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![], vec![area0()]),
            piste(
                "2",
                vec![],
                vec![area1_0(), area1_1(), area1_2(), area1_3()],
            ),
        ],
    );
    let segments = segments_follow_piste();
    let expected = vec![
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 33),
        ),
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 38),
        ),
    ];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn stay_on_piste_when_crossing_another_with_lines() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![line0()], vec![]),
            piste("2", vec![line1()], vec![]),
        ],
    );
    let segments = segments_follow_piste();
    let expected = vec![(
        Moving {
            piste_id: "2".to_string(),
            move_type: MoveType::Ski,
        },
        (0, 0),
    )];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn stay_on_piste_when_crossing_another_from_area_to_line() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![line0()], vec![]),
            piste(
                "2",
                vec![],
                vec![area1_0(), area1_1(), area1_2(), area1_3()],
            ),
        ],
    );
    let segments = segments_follow_piste();
    let expected = vec![
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 33),
        ),
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 38),
        ),
    ];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn stay_on_piste_when_crossing_another_from_line_to_area() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![], vec![area0()]),
            piste("2", vec![line1()], vec![]),
        ],
    );
    let segments = segments_follow_piste();
    let expected = vec![(
        Moving {
            piste_id: "2".to_string(),
            move_type: MoveType::Ski,
        },
        (0, 0),
    )];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

fn segments_leave_for_short_time() -> Vec<TrackSegment> {
    vec![segment(&[
        (6.5260963, 45.3125588),
        (6.5262519, 45.3126741),
        (6.5264425, 45.3128468),
        (6.5266085, 45.3130066),
        (6.5267759, 45.3131474),
        (6.5267665, 45.3133135),
        (6.5269201, 45.3134453),
        (6.5271804, 45.3135333),
        (6.5273692, 45.3136859),
        (6.5275999, 45.3138751),
        (6.5276736, 45.3140696),
        (6.5276104, 45.3141994),
        (6.5277122, 45.3142864),
        (6.5278658, 45.3142588),
        (6.5279933, 45.3142539),
        (6.5282917, 45.3143203),
    ])]
}

#[test]
#[named]
fn stay_on_piste_when_leaving_for_a_short_time_with_areas() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![], vec![area0()])]);
    let segments = segments_leave_for_short_time();
    let expected = vec![(
        Moving {
            piste_id: "1".to_string(),
            move_type: MoveType::Ski,
        },
        (0, 0),
    )];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn stay_on_piste_when_leaving_for_a_short_time_with_lines() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![line0()], vec![])]);
    let segments = segments_leave_for_short_time();
    let expected = vec![(
        Moving {
            piste_id: "1".to_string(),
            move_type: MoveType::Ski,
        },
        (0, 0),
    )];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}
