use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    process::CandidateFactory,
    simple_candidate::{
        Constraint, ConstraintLimit, ConstraintType, SimpleCandidateFactory,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MoveType {
    Ski,
    Wait,
    Climb,
    Traverse,
}

pub fn get_move_candidates() -> HashMap<MoveType, Box<dyn CandidateFactory>> {
    [
        (
            MoveType::Ski,
            SimpleCandidateFactory::new(vec![
                Constraint::new(
                    ConstraintType::Speed,
                    Some(5.0),
                    None,
                    ConstraintLimit::Distance,
                    100.0,
                ),
                Constraint::new(
                    ConstraintType::Inclination,
                    None,
                    Some(-0.05),
                    ConstraintLimit::Distance,
                    200.0,
                ),
            ]),
        ),
        (
            MoveType::Wait,
            SimpleCandidateFactory::new(vec![Constraint::new(
                ConstraintType::Speed,
                None,
                Some(1.0),
                ConstraintLimit::Time,
                20.0,
            )]),
        ),
        (
            MoveType::Climb,
            SimpleCandidateFactory::new(vec![
                Constraint::new(
                    ConstraintType::Speed,
                    Some(0.1),
                    Some(2.0),
                    ConstraintLimit::Time,
                    20.0,
                ),
                Constraint::new(
                    ConstraintType::Inclination,
                    Some(0.05),
                    None,
                    ConstraintLimit::Time,
                    20.0,
                ),
            ]),
        ),
        (
            MoveType::Traverse,
            SimpleCandidateFactory::new(vec![
                Constraint::new(
                    ConstraintType::Speed,
                    Some(0.5),
                    Some(5.0),
                    ConstraintLimit::Time,
                    20.0,
                ),
                Constraint::new(
                    ConstraintType::Inclination,
                    Some(-0.1),
                    Some(0.1),
                    ConstraintLimit::Distance,
                    100.0,
                ),
            ]),
        ),
    ]
    .into()
}
