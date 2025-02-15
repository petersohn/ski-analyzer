use super::{
    process::Candidate,
    simple_candidate::{
        Constraint, ConstraintLimit, ConstraintType, SimpleCandidate,
    },
};
use crate::utils::test_util::{make_gpx, segment, Init};

use gpx::TrackSegment;
use rstest::{fixture, rstest};
use time::{Duration, OffsetDateTime};

fn time(seconds: f64) -> OffsetDateTime {
    OffsetDateTime::UNIX_EPOCH + Duration::seconds_f64(seconds)
}

fn add_time_and_ele(segment: &mut TrackSegment, data: &[(f64, f64)]) {
    for (wp, (secs, ele)) in segment.points.iter_mut().zip(data) {
        wp.time = Some(time(*secs).into());
        wp.elevation = Some(*ele);
    }
}

#[fixture]
fn segment0() -> TrackSegment {
    segment(&[
        (6.5218066, 45.3169126),
        (6.5217617, 45.3169933),
        (6.521729, 45.3170806),
        (6.521727, 45.3171681),
        (6.5217359, 45.3172596),
        (6.5217475, 45.3173509),
        (6.521757, 45.3174405),
        (6.5217614, 45.3175342),
        (6.5217512, 45.317622),
        (6.5217575, 45.3177157),
        (6.5217614, 45.3178026),
        (6.5217594, 45.3178962),
        (6.5217818, 45.3179876),
    ])
}

fn run_test(
    mut candidate: SimpleCandidate,
    segment: &TrackSegment,
    expected_values: &[Option<bool>],
) {
    for i in 0..expected_values.len() {
        let wp0 = &segment.points[i];
        let wp1 = &segment.points[i + 1];
        let actual = candidate.add_line(wp0, wp1);
        assert_eq!(actual, expected_values[i], "{wp0:#?} -> {wp1:#?}");
    }
}

#[rstest]
fn speed_min_ok(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (4.0, 0.0),
            (5.0, 0.0),
        ],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Speed,
        Some(5.0),
        None,
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(
        candidate,
        &segment0,
        &[None, None, Some(true), Some(true), Some(true)],
    );
}

#[rstest]
fn speed_min_nok(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Speed,
        Some(15.0),
        None,
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(candidate, &segment0, &[None, None, Some(false)]);
}

#[rstest]
fn speed_max_ok(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (4.0, 0.0),
            (5.0, 0.0),
        ],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Speed,
        None,
        Some(12.0),
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(
        candidate,
        &segment0,
        &[None, None, Some(true), Some(true), Some(true)],
    );
}

#[rstest]
fn speed_max_nok(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Speed,
        None,
        Some(6.0),
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(candidate, &segment0, &[None, None, Some(false)]);
}

#[rstest]
fn speed_minmax_ok(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (4.0, 0.0),
            (5.0, 0.0),
        ],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Speed,
        Some(8.0),
        Some(12.0),
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(
        candidate,
        &segment0,
        &[None, None, Some(true), Some(true), Some(true)],
    );
}

#[rstest]
fn speed_minmax_too_slow(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Speed,
        Some(12.0),
        Some(20.0),
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(candidate, &segment0, &[None, None, Some(false)]);
}

#[rstest]
fn speed_minmax_too_fast(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Speed,
        Some(5.0),
        Some(8.0),
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(candidate, &segment0, &[None, None, Some(false)]);
}

#[rstest]
fn speed_minmax_ok_by_average(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (4.0, 0.0),
            (5.0, 0.0),
            (6.5, 0.0), // 40/4.5 = 8.8...
            (7.0, 0.0),
            (7.5, 0.0), // 40/3.5 ~ 11.4
            (8.5, 0.0),
            (10.0, 0.0),
            (11.0, 0.0),
            (12.0, 0.0),
        ],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Speed,
        Some(8.0),
        Some(12.0),
        ConstraintLimit::Distance,
        35.0,
    )]);
    run_test(
        candidate,
        &segment0,
        &[
            None,
            None,
            None,
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
        ],
    );
}

#[rstest]
fn speed_minmax_nok_by_average(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (4.0, 0.0),
            (5.0, 0.0),
            (6.5, 0.0),
            (8.0, 0.0),
            (9.5, 0.0),
        ],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Speed,
        Some(8.0),
        Some(12.0),
        ConstraintLimit::Distance,
        35.0,
    )]);
    run_test(
        candidate,
        &segment0,
        &[
            None,
            None,
            None,
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(false),
        ],
    );
}

#[rstest]
fn inclination_minmax_ok(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[
            (0.0, 100.0),
            (1.0, 99.0),
            (2.0, 98.0),
            (3.0, 97.0),
            (4.0, 96.0),
            (5.0, 95.0),
        ],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Inclination,
        Some(-0.12),
        Some(-0.08),
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(
        candidate,
        &segment0,
        &[None, None, Some(true), Some(true), Some(true)],
    );
}

#[rstest]
fn inclination_minmax_too_steep(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[(0.0, 100.0), (1.0, 98.0), (2.0, 96.0), (3.0, 94.0)],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Inclination,
        Some(-0.12),
        Some(-0.08),
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(candidate, &segment0, &[None, None, Some(false)]);
}

#[rstest]
fn inclination_minmax_not_steep_enough(mut segment0: TrackSegment) {
    add_time_and_ele(
        &mut segment0,
        &[(0.0, 100.0), (1.0, 99.50), (2.0, 99.0), (3.0, 98.5)],
    );
    let candidate = SimpleCandidate::new([Constraint::new(
        ConstraintType::Inclination,
        Some(-0.12),
        Some(-0.08),
        ConstraintLimit::Distance,
        25.0,
    )]);
    run_test(candidate, &segment0, &[None, None, Some(false)]);
}
