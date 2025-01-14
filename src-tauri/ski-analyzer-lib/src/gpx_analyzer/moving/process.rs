use std::collections::HashMap;

use gpx::Waypoint;

use super::super::segments::Segments;
use super::super::Activity;
use super::MoveType;
use crate::error::Result;
use crate::ski_area::SkiArea;
use crate::utils::cancel::CancellationToken;

pub trait Candidate {
    fn add_point(&mut self, wp: Waypoint) -> bool;
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
    ski_area: &'s SkiArea,
    segments: Segments,
    move_types: &HashMap<MoveType, Box<dyn CandidateFactory>>,
) -> Result<Vec<Activity>> {
    let mut result = Vec::new();

    let mut candidates = create_candidates(move_types);

    let current_route = segments.process(
        |current_route, route_segment, point, mut coordinate| {
            cancel.check()?;
            Ok(())
        },
    );
    Ok(result)
}
