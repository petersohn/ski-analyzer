use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    process::{Candidate, CandidateFactory},
    //simple_candidate::{
    //    Constraint, ConstraintLimit, ConstraintType, SimpleCandidateFactory,
    //},
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MoveType {
    Ski,
    Wait,
    Climb,
    Traverse,
}

struct DummyCandidate {}

impl Candidate for DummyCandidate {
    fn add_line(
        &mut self,
        _wp0: &gpx::Waypoint,
        _wp1: &gpx::Waypoint,
    ) -> Option<bool> {
        Some(true)
    }
}

struct DummyCandidateFactory {}

impl CandidateFactory for DummyCandidateFactory {
    fn create_candidate(&self) -> Box<dyn super::process::Candidate> {
        Box::new(DummyCandidate {})
    }
}

pub fn get_move_candidates() -> HashMap<MoveType, Box<dyn CandidateFactory>> {
    let factory: Box<dyn CandidateFactory> = Box::new(DummyCandidateFactory {});
    [(MoveType::Ski, factory)]
        //[
        //    (
        //        MoveType::Ski,
        //        SimpleCandidateFactory::new(vec![
        //            Constraint::new(
        //                ConstraintType::Speed,
        //                Some(5.0),
        //                None,
        //                ConstraintLimit::Distance,
        //                100.0,
        //            ),
        //            Constraint::new(
        //                ConstraintType::Inclination,
        //                None,
        //                Some(-0.05),
        //                ConstraintLimit::Distance,
        //                200.0,
        //            ),
        //        ]),
        //    ),
        //    (
        //        MoveType::Wait,
        //        SimpleCandidateFactory::new(vec![Constraint::new(
        //            ConstraintType::Speed,
        //            None,
        //            Some(1.0),
        //            ConstraintLimit::Time,
        //            20.0,
        //        )]),
        //    ),
        //    (
        //        MoveType::Climb,
        //        SimpleCandidateFactory::new(vec![
        //            Constraint::new(
        //                ConstraintType::Speed,
        //                Some(0.1),
        //                Some(2.0),
        //                ConstraintLimit::Time,
        //                20.0,
        //            ),
        //            Constraint::new(
        //                ConstraintType::Inclination,
        //                Some(0.05),
        //                None,
        //                ConstraintLimit::Time,
        //                20.0,
        //            ),
        //        ]),
        //    ),
        //    (
        //        MoveType::Traverse,
        //        SimpleCandidateFactory::new(vec![
        //            Constraint::new(
        //                ConstraintType::Speed,
        //                Some(0.5),
        //                Some(5.0),
        //                ConstraintLimit::Time,
        //                20.0,
        //            ),
        //            Constraint::new(
        //                ConstraintType::Inclination,
        //                Some(-0.1),
        //                Some(0.1),
        //                ConstraintLimit::Distance,
        //                100.0,
        //            ),
        //        ]),
        //    ),
        //]
        .into()
}
