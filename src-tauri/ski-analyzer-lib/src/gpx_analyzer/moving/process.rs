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

pub fn process_moves(
    cancel: &CancellationToken,
    segments: &Segments,
    move_types: &HashMap<MoveType, Box<dyn CandidateFactory>>,
) -> Result<Vec<(MoveType, SegmentCoordinate)>> {
    let mut result: Vec<(MoveType, SegmentCoordinate)> = Vec::new();

    let mut candidates = create_candidates(move_types);
    let mut is_committed = true;

    let mut coordinate = segments.begin_coord();
    while coordinate != segments.end_coord() {
        cancel.check()?;
        let point = segments.get(coordinate).unwrap();
        let mut to_remove: Vec<MoveType> = Vec::new();
        for (move_type, candidate) in &mut candidates {
            if !candidate.add_point(point) {
                to_remove.push(*move_type);
            }
        }
        for move_type in &to_remove {
            candidates.remove(move_type);
        }
        eprintln!(
            "{:?}: {:?} -> {:?}",
            coordinate,
            to_remove,
            candidates.keys().map(|x| *x).collect::<Vec<MoveType>>()
        );
        if candidates.is_empty() {
            eprintln!("  -> {:?}", to_remove.first().unwrap());
            result.push((*to_remove.first().unwrap(), coordinate));
            candidates = create_candidates(move_types);
            is_committed = true;
        } else {
            is_committed = false;
            coordinate = segments.next_coord(coordinate);
        }
    }

    if !is_committed {
        result.push((
            candidates.into_iter().next().unwrap().0,
            segments.end_coord(),
        ));
    }

    Ok(result)
}
