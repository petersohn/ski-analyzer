use std::collections::HashMap;
use std::mem::take;

use gpx::Waypoint;

use super::super::segments::Segments;
use super::MoveType;
use crate::error::Result;
use crate::gpx_analyzer::SegmentCoordinate;
use crate::utils::cancel::CancellationToken;

pub trait Candidate {
    fn add_point(&mut self, wp: &Waypoint) -> bool;
}

pub trait CandidateFactory {
    fn create_candidate(&self) -> Box<dyn Candidate>;
}

fn create_candidates(
    move_types: &HashMap<MoveType, Box<dyn CandidateFactory>>,
) -> HashMap<MoveType, Box<dyn Candidate>> {
    move_types
        .iter()
        .map(|(t, f)| (*t, f.create_candidate()))
        .collect()
}

pub fn process_moves<'s>(
    cancel: &CancellationToken,
    segments: Segments,
    move_types: &HashMap<MoveType, Box<dyn CandidateFactory>>,
) -> Result<Vec<(MoveType, SegmentCoordinate)>> {
    let mut result: Vec<(MoveType, SegmentCoordinate)> = Vec::new();

    let mut candidates = create_candidates(move_types);

    let current_route = segments.process(
        |_current_route, _route_segment, point, coordinate| {
            cancel.check()?;
            let mut to_remove: Vec<MoveType> = Vec::new();
            for (move_type, candidate) in &mut candidates {
                if !candidate.add_point(point) {
                    to_remove.push(*move_type);
                }
            }
            for move_type in &to_remove {
                candidates.remove(move_type);
            }
            if candidates.is_empty() {
                result.push((*to_remove.first().unwrap(), coordinate));
            }
            Ok(())
        },
    );

    //if !current_route.0.is_empty() {
    //    result.push((candidates.into_keys().next(), ));
    //}

    Ok(result)
}
