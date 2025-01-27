use std::collections::HashMap;
use std::mem::take;

use gpx::Waypoint;

use super::super::segments::Segments;
use super::{move_type, MoveType};
use crate::error::Result;
use crate::gpx_analyzer::SegmentCoordinate;
use crate::utils::cancel::CancellationToken;

// Some(true) -> can commit
// Some(false) -> should drop
// None -> cannot commit, but should not drop
pub trait Candidate {
    fn add_point(&mut self, wp: &Waypoint) -> Option<bool>;
}

pub trait CandidateFactory {
    fn create_candidate(&self) -> Box<dyn Candidate>;
}

fn fill_candidates(
    move_types: &HashMap<MoveType, Box<dyn CandidateFactory>>,
    coordinate: SegmentCoordinate,
    candidates: &mut HashMap<MoveType, (SegmentCoordinate, Box<dyn Candidate>)>,
) {
    for (move_type, factory) in move_types {
        candidates
            .entry(*move_type)
            .or_insert_with(|| (coordinate, factory.create_candidate()));
    }
}

pub fn process_moves(
    cancel: &CancellationToken,
    segments: &Segments,
    move_types: &HashMap<MoveType, Box<dyn CandidateFactory>>,
) -> Result<Vec<(Option<MoveType>, SegmentCoordinate)>> {
    let mut result = Vec::new();

    let mut candidates: HashMap<
        MoveType,
        (SegmentCoordinate, Box<dyn Candidate>),
    > = HashMap::new();
    fill_candidates(move_types, (0, 0), &mut candidates);
    let mut is_bad = false;
    let mut was_bad = false;
    let mut previous_to_commit: Option<(SegmentCoordinate, MoveType)> = None;

    let mut coordinate = segments.begin_coord();
    while coordinate != segments.end_coord() {
        cancel.check()?;
        eprintln!("{coordinate:?}");
        let point = segments.get(coordinate).unwrap();
        let mut to_remove: Vec<MoveType> = Vec::new();
        let mut to_commit: Option<(SegmentCoordinate, MoveType)> = None;
        for (move_type, (from, candidate)) in &mut candidates {
            let res = candidate.add_point(point);
            eprintln!("  {move_type:?} -> {res:?}");
            match res {
                None => (),
                Some(false) => to_remove.push(*move_type),
                Some(true) => {
                    if to_commit.is_none() {
                        to_commit = Some((*from, *move_type));
                    }
                }
            }
        }
        for move_type in &to_remove {
            candidates.remove(move_type);
        }
        if candidates.is_empty() {
            match take(&mut previous_to_commit) {
                None => {
                    eprintln!("  bad");
                    is_bad = true;
                    coordinate = segments.next_coord(coordinate);
                }
                Some((from, move_type)) => {
                    eprintln!("  good was_bad={was_bad}");
                    if was_bad {
                        result.push((None, from));
                        was_bad = false;
                    }
                    result.push((Some(move_type), coordinate));
                }
            };
            fill_candidates(move_types, coordinate, &mut candidates);
        } else {
            eprintln!("  good is_bad={is_bad} was_bad={was_bad} to_commit={to_commit:?}");

            if is_bad {
                was_bad = true;
                is_bad = false;
            }

            if was_bad && to_commit.is_none() {
                fill_candidates(move_types, coordinate, &mut candidates);
            }

            previous_to_commit = to_commit;
            coordinate = segments.next_coord(coordinate);
        }
    }

    match take(&mut previous_to_commit) {
        None => {
            result.push((None, coordinate));
        }
        Some((from, move_type)) => {
            if was_bad {
                result.push((None, from));
            }
            result.push((Some(move_type), coordinate));
        }
    };

    Ok(result)
}
