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
        (MoveType::Ski, SimpleCandidateFactory::new(vec![])),
        (MoveType::Wait, SimpleCandidateFactory::new(vec![])),
        (MoveType::Climb, SimpleCandidateFactory::new(vec![])),
        (MoveType::Traverse, SimpleCandidateFactory::new(vec![])),
    ]
    .into()
}
