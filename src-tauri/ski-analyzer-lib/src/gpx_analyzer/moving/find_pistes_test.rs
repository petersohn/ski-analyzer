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
};
use piste_data::{area0, area1_0, area1_1, area1_2, area1_3, line0, line1};
use segment_data::{
    segments0, segments_enter_leave, segments_follow_piste,
    segments_leave_for_short_time, segments_multiple0, segments_multiple1,
};

mod piste_data;
mod segment_data;

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

#[test]
#[named]
fn multiple_segments() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![line0()], vec![]),
            piste("2", vec![line1()], vec![]),
        ],
    );
    let segments = segments_multiple0();
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
            (1, 0),
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
fn multiple_segments_to_outside_piste() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![], vec![area0()])]);
    let segments = segments_multiple1();
    let expected = vec![
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "".to_string(),
                move_type: MoveType::Ski,
            },
            (1, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (1, 3),
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
