use std::collections::{hash_map, HashMap};
use std::mem::take;

use gpx::Waypoint;

use super::super::segments::Segments;
use super::MoveType;
use crate::error::Result;
use crate::gpx_analyzer::SegmentCoordinate;
use crate::utils::cancel::CancellationToken;

// Some(true) -> can commit
// Some(false) -> should drop
// None -> cannot commit, but should not drop
pub trait Candidate {
    fn add_line(&mut self, wp0: &Waypoint, wp1: &Waypoint) -> Option<bool>;
}

pub trait CandidateFactory {
    fn create_candidate(&self) -> Box<dyn Candidate>;
}

struct FinishedCandidate {
    move_type: MoveType,
    min: SegmentCoordinate,
    max: SegmentCoordinate,
}

struct Process<'a> {
    move_types: &'a HashMap<MoveType, Box<dyn CandidateFactory>>,
    candidates: HashMap<MoveType, (SegmentCoordinate, Box<dyn Candidate>)>,
    can_finish: HashMap<MoveType, SegmentCoordinate>,
    finished_candidates: Vec<FinishedCandidate>,
    result: Vec<(Option<MoveType>, SegmentCoordinate)>,
    last_commit: SegmentCoordinate,
}

impl<'a> Process<'a> {
    fn new(
        move_types: &'a HashMap<MoveType, Box<dyn CandidateFactory>>,
    ) -> Self {
        Self {
            move_types,
            candidates: HashMap::new(),
            can_finish: HashMap::new(),
            finished_candidates: Vec::new(),
            result: Vec::new(),
            last_commit: (0, 0),
        }
    }

    fn fill(&mut self, coordinate: SegmentCoordinate) {
        for (move_type, factory) in self.move_types {
            self.candidates
                .entry(*move_type)
                .or_insert_with(|| (coordinate, factory.create_candidate()));
        }
    }

    fn add_point(
        &mut self,
        coordinate: SegmentCoordinate,
        wp0: &Waypoint,
        wp1: &Waypoint,
    ) -> (bool, String) {
        let mut to_remove: Vec<MoveType> = Vec::new();
        let mut comment = String::new();
        for (move_type, (from, candidate)) in &mut self.candidates {
            let res = candidate.add_line(wp0, wp1);
            match res {
                None => (),
                Some(false) => to_remove.push(*move_type),
                Some(true) => {
                    self.can_finish.entry(*move_type).or_insert(*from);
                }
            };
            comment.push_str(&format!("{move_type:?} -> {res:?}\n"));
        }

        for move_type in &to_remove {
            self.candidates.remove(move_type);
            self.finish(*move_type, coordinate);
        }

        (!to_remove.is_empty(), comment)
    }

    fn finish(&mut self, move_type: MoveType, coordinate: SegmentCoordinate) {
        if let hash_map::Entry::Occupied(entry) =
            self.can_finish.entry(move_type)
        {
            let min = entry.remove();
            let max = coordinate;
            self.finished_candidates.push(FinishedCandidate {
                move_type,
                min,
                max,
            });
        }
    }

    fn finish_all(&mut self, coordinate: SegmentCoordinate) {
        for move_type in self
            .candidates
            .keys()
            .map(|c| *c)
            .collect::<Vec<MoveType>>()
        {
            self.finish(move_type, coordinate);
        }
        self.candidates.clear();
    }

    fn commit(&mut self) -> String {
        if self.finished_candidates.is_empty() {
            return String::new();
        }

        let mut finished_candidates = take(&mut self.finished_candidates);

        finished_candidates.sort_by_key(|c| std::cmp::Reverse(c.min));

        let mut comment = String::new();

        let mut push = |x, coord| {
            comment.push_str(&format!("commit {coord:?} {x:?}\n"));
            self.result.push((x, coord));
        };

        while !finished_candidates.is_empty() {
            let first_coord = finished_candidates.last().unwrap().min;
            let idx =
                finished_candidates.partition_point(|c| c.min != first_coord);
            let to_commit = finished_candidates
                .drain(idx..)
                .max_by_key(|c| c.max)
                .unwrap();
            if to_commit.min != self.last_commit {
                push(None, to_commit.min);
            }
            push(Some(to_commit.move_type), to_commit.max);
            self.last_commit = to_commit.max;
            finished_candidates = finished_candidates
                .into_iter()
                .filter(|c| c.max > self.last_commit)
                .collect();
            for c in &mut finished_candidates {
                if c.min < self.last_commit {
                    c.min = self.last_commit;
                }
            }
        }

        comment
    }

    fn should_commit(&self, coordinate: SegmentCoordinate) -> bool {
        self.candidates.values().all(|(c, _)| *c == coordinate)
    }
}

pub fn process_moves(
    cancel: &CancellationToken,
    segments: &mut Segments,
    move_types: &HashMap<MoveType, Box<dyn CandidateFactory>>,
) -> Result<Vec<(Option<MoveType>, SegmentCoordinate)>> {
    let mut process = Process::new(move_types);
    let mut prev: Option<&Waypoint> = None;
    let mut comments: HashMap<SegmentCoordinate, String> = HashMap::new();

    for (coordinate, point) in &*segments {
        cancel.check()?;
        if coordinate.1 == 0 {
            process.finish_all(coordinate);
            comments.insert(coordinate, process.commit());
            prev = Some(point);
            process.fill(coordinate);
            continue;
        }

        process.fill(coordinate);
        let (was_finished, mut comment) =
            process.add_point(coordinate, prev.unwrap(), point);
        let should_commit = process.should_commit(coordinate);
        comment.push_str(&format!(
            "was_finished={was_finished}, should_commit={should_commit}\n"
        ));

        if was_finished && should_commit {
            comment.push_str(&process.commit());
        }

        comments.insert(coordinate, comment);
        prev = Some(point);
    }

    let mut coordinate = segments.begin_coord();
    while coordinate != segments.end_coord() {
        if let Some(comment) = comments.get_mut(&coordinate) {
            segments.get_mut(coordinate).as_mut().unwrap().comment =
                Some(format!("{coordinate:?}\n{comment}"));
        }
        coordinate = segments.next_coord(coordinate);
    }

    let end = segments.end_coord();
    process.finish_all(end);
    process.commit();
    if process.last_commit != end {
        process.result.push((None, end));
    }
    Ok(process.result)
}
