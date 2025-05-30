use super::{Activity, ActivityType, SegmentCoordinate, Segments};
use crate::error::Result;
use crate::ski_area::SkiArea;
use crate::utils::cancel::CancellationToken;

use find_pistes::find_pistes;
use move_type::get_move_candidates;
use process::process_moves;

use serde::{Deserialize, Serialize};

mod find_pistes;
mod move_type;
mod process;
mod simple_candidate;

#[cfg(test)]
mod find_pistes_test;
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

fn commit_moves(
    segments: &mut Segments,
    coords_with_pistes: Vec<(Moving, SegmentCoordinate)>,
) -> Vec<Activity> {
    segments.commit(None, |_segments| {
        coords_with_pistes.into_iter().map(|(moving, coord)| {
            let activity_type = ActivityType::Moving(moving);
            (activity_type, coord)
        })
    })
}

pub fn find_moves<'s>(
    cancel: &CancellationToken,
    ski_area: &'s SkiArea,
    mut segments: Segments,
) -> Result<Vec<Activity>> {
    let move_coords =
        process_moves(cancel, &mut segments, &get_move_candidates())?;
    let coords_with_pistes =
        find_pistes(cancel, ski_area, &segments, move_coords)?;

    let moves = commit_moves(&mut segments, coords_with_pistes);
    Ok(moves)
}
