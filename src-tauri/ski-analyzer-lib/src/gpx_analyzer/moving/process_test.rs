use std::collections::HashMap;

use gpx::Waypoint;

use super::process::{process_moves, Candidate, CandidateFactory};
use super::MoveType;
use crate::gpx_analyzer::test_util::wp;
use crate::gpx_analyzer::Segments;
use crate::utils::cancel::CancellationToken;

#[derive(Clone, Copy)]
struct TestCandidate {
    min: f64,
    max: f64,
}

impl Candidate for TestCandidate {
    fn add_point(&mut self, wp: &Waypoint) -> bool {
        let p = wp.point();
        p.x() >= self.min && p.x() <= self.max
    }
}

struct TestCandidateFactory {
    candidate: TestCandidate,
}

impl TestCandidateFactory {
    fn new(min: f64, max: f64) -> Self {
        Self {
            candidate: TestCandidate { min, max },
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

#[test]
fn single_candidate() {
    let segments = Segments::new(vec![
        vec![wp(1.0, 0.0, None), wp(2.0, 0.0, None), wp(3.0, 0.0, None)],
        vec![wp(4.0, 0.0, None), wp(5.0, 0.0, None), wp(6.0, 0.0, None)],
        vec![wp(7.0, 0.0, None), wp(8.0, 0.0, None), wp(9.0, 0.0, None)],
    ]);
    let move_types = cfs(&[
        (MoveType::Ski, 1.0, 4.0),
        (MoveType::Wait, 5.0, 5.0),
        (MoveType::Climb, 6.0, 100.0),
    ]);
    let actual =
        process_moves(&CancellationToken::new(), &segments, &move_types)
            .unwrap();
    let expected = vec![
        (MoveType::Ski, (1, 1)),
        (MoveType::Wait, (1, 2)),
        (MoveType::Climb, (3, 0)),
    ];
    assert_eq!(actual, expected);
}
