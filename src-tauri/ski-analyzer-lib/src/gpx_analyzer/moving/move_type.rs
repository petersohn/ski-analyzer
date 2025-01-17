use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MoveType {
    Ski,
    Wait,
    Climb,
    Traverse,
}
