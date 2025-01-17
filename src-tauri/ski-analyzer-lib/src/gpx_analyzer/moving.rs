use serde::{Deserialize, Serialize};

mod move_type;
mod process;

#[cfg(test)]
mod process_test;

pub use move_type::MoveType;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Moving {
    pub ski_type: MoveType,
    pub piste_id: String,
}
