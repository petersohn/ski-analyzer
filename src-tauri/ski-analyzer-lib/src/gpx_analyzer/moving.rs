use super::{Activity, ActivityType, Segments};
use crate::error::Result;
use crate::ski_area::SkiArea;
use crate::utils::cancel::CancellationToken;

use move_type::get_move_candidates;
use process::process_moves;

use serde::{Deserialize, Serialize};

mod find_pistes;
mod move_type;
mod process;
mod simple_candidate;

#[cfg(test)]
mod process_test;
#[cfg(test)]
mod simple_candidate_test;

pub use move_type::MoveType;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Moving {
    pub move_type: MoveType,
    pub piste_id: String,
}

pub fn find_moves<'s>(
    cancel: &CancellationToken,
    _ski_area: &'s SkiArea,
    mut segments: Segments,
) -> Result<Vec<Activity>> {
    let move_coords =
        process_moves(cancel, &mut segments, &get_move_candidates())?;

    let moves = segments.commit(None, |_segments| {
        move_coords.into_iter().map(|(move_type, coord)| {
            let activity_type =
                move_type.map_or_else(ActivityType::default, |mt| {
                    ActivityType::Moving(Moving {
                        move_type: mt,
                        piste_id: "".to_string(),
                    })
                });
            (activity_type, coord)
        })
    });
    Ok(moves)
}
