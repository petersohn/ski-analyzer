use std::collections::HashMap;

use gpx::Waypoint;

use super::process::{process_moves, Candidate, CandidateFactory};
use super::MoveType;
use crate::gpx_analyzer::test_util::wp;
use crate::gpx_analyzer::Segments;
use crate::utils::cancel::CancellationToken;

#[derive(Clone, Copy)]
struct TestCandidate {
    min_none: f64,
    min: f64,
    max: f64,
}

impl Candidate for TestCandidate {
    fn add_line(&mut self, _wp0: &Waypoint, wp1: &Waypoint) -> Option<bool> {
        let p = wp1.point();
        if p.x() >= self.min_none && p.x() < self.min {
            None
        } else {
            Some(p.x() >= self.min && p.x() <= self.max)
        }
    }
}

struct TestCandidateFactory {
    candidate: TestCandidate,
}

impl TestCandidateFactory {
    fn new(min: f64, max: f64) -> Self {
        Self {
            candidate: TestCandidate {
                min_none: min,
                min,
                max,
            },
        }
    }

    fn with_none(min_none: f64, min: f64, max: f64) -> Self {
        Self {
            candidate: TestCandidate { min_none, min, max },
        }
    }
}

impl CandidateFactory for TestCandidateFactory {
    fn create_candidate(&self) -> Box<dyn Candidate> {
        Box::new(self.candidate)
    }
}

fn cfs(
    input: &[(MoveType, f64, f64)],
) -> HashMap<MoveType, Box<dyn CandidateFactory>> {
    input
        .iter()
        .map(|(move_type, min, max)| {
            let cf: Box<dyn CandidateFactory> =
                Box::new(TestCandidateFactory::new(*min, *max));
            (*move_type, cf)
        })
        .collect()
}

fn cfs_none(
    input: &[(MoveType, f64, f64, f64)],
) -> HashMap<MoveType, Box<dyn CandidateFactory>> {
    input
        .iter()
        .map(|(move_type, min_none, min, max)| {
            let cf: Box<dyn CandidateFactory> = Box::new(
                TestCandidateFactory::with_none(*min_none, *min, *max),
            );
            (*move_type, cf)
        })
        .collect()
}

#[test]
fn single_candidate() {
    let segments = Segments::new(vec![vec![
        wp(1.0, 0.0, None),
        wp(2.0, 0.0, None),
        wp(3.0, 0.0, None),
        wp(4.0, 0.0, None),
        wp(5.0, 0.0, None),
        wp(6.0, 0.0, None),
        wp(7.0, 0.0, None),
        wp(8.0, 0.0, None),
        wp(9.0, 0.0, None),
    ]]);
    let move_types = cfs(&[
        (MoveType::Ski, 1.0, 4.0),
        (MoveType::Wait, 5.0, 5.0),
        (MoveType::Climb, 6.0, 100.0),
    ]);
    let actual =
        process_moves(&CancellationToken::new(), &segments, &move_types)
            .unwrap();
    let expected = vec![
        (Some(MoveType::Ski), (0, 4)),
        (Some(MoveType::Wait), (0, 5)),
        (Some(MoveType::Climb), (1, 0)),
    ];
    assert_eq!(actual, expected);
}

#[test]
fn multiple_candidate() {
    let segments = Segments::new(vec![vec![
        wp(1.0, 0.0, None),
        wp(2.0, 0.0, None),
        wp(3.0, 0.0, None),
        wp(4.0, 0.0, None),
        wp(5.0, 0.0, None),
        wp(6.0, 0.0, None),
        wp(7.0, 0.0, None),
        wp(8.0, 0.0, None),
        wp(9.0, 0.0, None),
    ]]);
    let move_types = cfs(&[
        (MoveType::Ski, 1.0, 4.0),
        (MoveType::Wait, 1.0, 5.0),
        (MoveType::Climb, 3.0, 100.0),
    ]);
    let actual =
        process_moves(&CancellationToken::new(), &segments, &move_types)
            .unwrap();
    let expected = vec![
        (Some(MoveType::Wait), (0, 5)),
        (Some(MoveType::Climb), (1, 0)),
    ];
    assert_eq!(actual, expected);
}

#[test]
fn bad_begin() {
    let segments = Segments::new(vec![vec![
        wp(1.0, 0.0, None),
        wp(2.0, 0.0, None),
        wp(3.0, 0.0, None),
        wp(4.0, 0.0, None),
        wp(5.0, 0.0, None),
        wp(6.0, 0.0, None),
        wp(7.0, 0.0, None),
        wp(8.0, 0.0, None),
        wp(9.0, 0.0, None),
    ]]);
    let move_types = cfs(&[(MoveType::Ski, 5.0, 10.0)]);
    let actual =
        process_moves(&CancellationToken::new(), &segments, &move_types)
            .unwrap();
    let expected = vec![(None, (0, 4)), (Some(MoveType::Ski), (1, 0))];
    assert_eq!(actual, expected);
}

#[test]
fn bad_end() {
    let segments = Segments::new(vec![vec![
        wp(1.0, 0.0, None),
        wp(2.0, 0.0, None),
        wp(3.0, 0.0, None),
        wp(4.0, 0.0, None),
        wp(5.0, 0.0, None),
        wp(6.0, 0.0, None),
        wp(7.0, 0.0, None),
        wp(8.0, 0.0, None),
        wp(9.0, 0.0, None),
    ]]);
    let move_types = cfs(&[(MoveType::Ski, 1.0, 4.0)]);
    let actual =
        process_moves(&CancellationToken::new(), &segments, &move_types)
            .unwrap();
    let expected = vec![(Some(MoveType::Ski), (0, 4)), (None, (1, 0))];
    assert_eq!(actual, expected);
}

#[test]
fn bad_middle() {
    let segments = Segments::new(vec![vec![
        wp(1.0, 0.0, None),
        wp(2.0, 0.0, None),
        wp(3.0, 0.0, None),
        wp(4.0, 0.0, None),
        wp(5.0, 0.0, None),
        wp(6.0, 0.0, None),
        wp(7.0, 0.0, None),
        wp(8.0, 0.0, None),
        wp(9.0, 0.0, None),
    ]]);
    let move_types =
        cfs(&[(MoveType::Ski, 1.0, 4.0), (MoveType::Climb, 7.0, 10.0)]);
    let actual =
        process_moves(&CancellationToken::new(), &segments, &move_types)
            .unwrap();
    let expected = vec![
        (Some(MoveType::Ski), (0, 4)),
        (None, (0, 6)),
        (Some(MoveType::Climb), (1, 0)),
    ];
    assert_eq!(actual, expected);
}

#[test]
fn unknown_then_good() {
    let segments = Segments::new(vec![vec![
        wp(1.0, 0.0, None),
        wp(2.0, 0.0, None),
        wp(3.0, 0.0, None),
        wp(4.0, 0.0, None),
        wp(5.0, 0.0, None),
        wp(6.0, 0.0, None),
        wp(7.0, 0.0, None),
        wp(8.0, 0.0, None),
        wp(9.0, 0.0, None),
    ]]);
    let move_types = cfs_none(&[
        (MoveType::Ski, 1.0, 3.0, 10.0),
        (MoveType::Climb, 1.0, 1.0, 5.0),
    ]);
    let actual =
        process_moves(&CancellationToken::new(), &segments, &move_types)
            .unwrap();
    let expected = vec![(Some(MoveType::Ski), (1, 0))];
    assert_eq!(actual, expected);
}

#[test]
fn unknown_then_bad() {
    let segments = Segments::new(vec![vec![
        wp(1.0, 0.0, None),
        wp(2.0, 0.0, None),
        wp(3.0, 0.0, None),
        wp(4.0, 0.0, None),
        wp(5.0, 0.0, None),
        wp(6.0, 0.0, None),
        wp(7.0, 0.0, None),
        wp(8.0, 0.0, None),
        wp(9.0, 0.0, None),
    ]]);
    let move_types = cfs_none(&[
        (MoveType::Traverse, 0.0, 1.0, 2.0),
        (MoveType::Ski, 2.0, 5.0, 0.0),
        (MoveType::Climb, 4.0, 7.0, 10.0),
    ]);
    let actual =
        process_moves(&CancellationToken::new(), &segments, &move_types)
            .unwrap();
    let expected = vec![
        (Some(MoveType::Traverse), (0, 2)),
        (None, (0, 3)),
        (Some(MoveType::Climb), (1, 0)),
    ];
    assert_eq!(actual, expected);
}

#[test]
fn multiple_segments() {
    let segments = Segments::new(vec![
        vec![
            wp(1.0, 0.0, None),
            wp(2.0, 0.0, None),
            wp(3.0, 0.0, None),
            wp(4.0, 0.0, None),
        ],
        vec![
            wp(5.0, 0.0, None),
            wp(6.0, 0.0, None),
            wp(7.0, 0.0, None),
            wp(8.0, 0.0, None),
            wp(9.0, 0.0, None),
        ],
    ]);
    let move_types = cfs_none(&[(MoveType::Ski, 0.0, 1.0, 10.0)]);
    let actual =
        process_moves(&CancellationToken::new(), &segments, &move_types)
            .unwrap();
    let expected =
        vec![(Some(MoveType::Ski), (1, 0)), (Some(MoveType::Ski), (2, 0))];
    assert_eq!(actual, expected);
}
