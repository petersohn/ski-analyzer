use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::take;

use geo::{
    Closest, Distance, Haversine, HaversineClosestPoint, Intersects, Length,
    Line, Point, Rect,
};

use super::{MoveType, Moving, Segments};
use crate::error::Result;
use crate::gpx_analyzer::SegmentCoordinate;
use crate::ski_area::{Difficulty, Piste, SkiArea};
use crate::utils::cancel::CancellationToken;
use crate::utils::rect::expand_rect;

const MAX_DISTANCE_NORMAL: f64 = 20.0;
const MAX_DISTANCE_FREERIDE: f64 = 100.0;
const MAX_OUTSIDE_LENGTH: f64 = 50.0;

fn get_min_distance(piste: &Piste) -> f64 {
    match piste.metadata.difficulty {
        Difficulty::Freeride => MAX_DISTANCE_FREERIDE,
        _ => MAX_DISTANCE_NORMAL,
    }
}

fn distance_to_geometry<G: HaversineClosestPoint<f64>>(
    g: &G,
    p: &Point,
) -> Option<f64> {
    let closest = match g.haversine_closest_point(p) {
        Closest::Intersection(p) => p,
        Closest::SinglePoint(p) => p,
        Closest::Indeterminate => return None,
    };
    Some(Haversine::distance(*p, closest))
}

fn check_distance(piste: &Piste, point: &Point) -> (bool, f64) {
    let da = distance_to_geometry(&piste.data.areas, point);
    if da == Some(0.0) {
        return (true, 0.0);
    }

    let dl = distance_to_geometry(&piste.data.lines, point);

    if let Some(d) = dl {
        if d < get_min_distance(piste) {
            return (true, d);
        }
    }

    let d = match (da, dl) {
        (Some(d1), Some(d2)) => f64::min(d1, d2),
        (Some(d), None) => d,
        (None, Some(d)) => d,
        (None, None) => 0.0,
    };

    return (false, d);
}

struct BadRun {
    count: usize,
    length: f64,
    last_point: Point,
}

struct Candidate<'a> {
    piste: &'a Piste,
    begin_coord: SegmentCoordinate,
    end_coord: Option<SegmentCoordinate>,
    distances: Vec<f64>,
    bad_run: Option<BadRun>,
}

impl<'a> Candidate<'a> {
    fn new(
        piste: &'a Piste,
        begin_coord: SegmentCoordinate,
        distance: f64,
    ) -> Self {
        Self {
            piste,
            begin_coord,
            end_coord: None,
            distances: vec![distance],
            bad_run: None,
        }
    }

    fn finish(&mut self, coord: SegmentCoordinate) {
        let end = match &self.bad_run {
            None => coord,
            Some(run) => {
                let e = (coord.0, coord.1 - (self.distances.len() - run.count));
                self.distances.truncate(run.count);
                e
            }
        };
        eprintln!(
            "finish {}: {:?} - {end:?}",
            self.piste.metadata.name, self.begin_coord
        );
        self.end_coord = Some(end);
    }

    fn add_point(&mut self, coord: SegmentCoordinate, point: &Point) {
        if self.is_finished() {
            return;
        }

        let (is_ok, d) = check_distance(&self.piste, point);
        eprintln!("* {coord:?} {}: {is_ok:?}, {d}", self.piste.metadata.name);
        if is_ok {
            self.distances.push(d);
            self.bad_run = None;
            return;
        }

        if d == 0.0 || d > MAX_OUTSIDE_LENGTH {
            self.finish(coord);
            return;
        }

        let can_continue = match self.bad_run.as_mut() {
            Some(run) => {
                let line = Line::new(run.last_point, *point);
                run.length += line.length::<Haversine>();
                run.last_point = *point;
                run.length <= MAX_OUTSIDE_LENGTH
            }
            None => {
                self.bad_run = Some(BadRun {
                    count: self.distances.len(),
                    length: d,
                    last_point: *point,
                });
                true
            }
        };

        if can_continue {
            self.distances.push(d);
        } else {
            self.finish(coord);
        }
    }

    fn is_finished(&self) -> bool {
        self.end_coord.is_some()
    }
}

struct Candidates<'a> {
    ski_area: &'a SkiArea,
    candidates: HashMap<String, Vec<Candidate<'a>>>,
    bounding_rects: HashMap<String, Rect>,
    first_empty: Option<SegmentCoordinate>,
}

impl<'a> Candidates<'a> {
    fn new(ski_area: &'a SkiArea) -> Self {
        Self {
            ski_area,
            candidates: HashMap::new(),
            bounding_rects: ski_area
                .pistes
                .iter()
                .map(|(id, p)| {
                    let mut r = p.data.bounding_rect;
                    expand_rect(&mut r, get_min_distance(p));
                    (id.clone(), r)
                })
                .collect(),
            first_empty: None,
        }
    }

    fn commit(
        &mut self,
        move_type: MoveType,
        coord: SegmentCoordinate,
    ) -> Vec<(Moving, SegmentCoordinate)> {
        let mut candidates: Vec<(String, Candidate<'a>)> =
            take(&mut self.candidates)
                .into_iter()
                .map(|(id, cs)| -> Vec<(String, Candidate)> {
                    cs.into_iter().map(|c| (id.clone(), c)).collect()
                })
                .flatten()
                .collect();
        eprintln!("commit candidates={}", candidates.len());

        for (_, c) in &mut candidates {
            if c.end_coord.is_none() {
                c.end_coord = Some(coord);
            }
        }

        candidates.sort_by(|(_, c1), (_, c2)| {
            c1.begin_coord
                .cmp(&c2.begin_coord)
                .then_with(|| c2.end_coord.unwrap().cmp(&c1.end_coord.unwrap()))
                .reverse()
        });

        let mut result = Vec::new();
        let mut push = |piste_id, begin| {
            result.push((
                Moving {
                    move_type,
                    piste_id,
                },
                begin,
            ))
        };

        let mut previous_begin: Option<SegmentCoordinate> = None;
        let mut current = candidates.pop();
        while let Some((piste_id, candidate)) = take(&mut current) {
            let begin = previous_begin.unwrap_or(candidate.begin_coord);
            let end = candidate.end_coord.unwrap();

            if begin >= end {
                current = candidates.pop();
                continue;
            }

            push(piste_id, begin);
            self.first_empty = Some(end);

            let mut possible_next = Vec::new();
            while candidates
                .last()
                .map_or(false, |(_, c)| c.begin_coord < end)
            {
                possible_next.push(candidates.pop().unwrap());
            }

            possible_next.sort_by_key(|(_, c)| c.end_coord.unwrap());
            let next = possible_next.pop();

            if next
                .as_ref()
                .map_or(true, |(_, c)| c.end_coord.unwrap() < end)
            {
                previous_begin = None;
                current = candidates.pop();
                if let Some((_, c)) = current.as_ref() {
                    if c.begin_coord > end {
                        push(String::new(), end);
                    }
                }
                continue;
            }

            let candidate2 = &next.as_ref().unwrap().1;
            let mut next_begin = None;
            for (i, (d_next, d_current)) in
                candidate2
                    .distances
                    .iter()
                    .zip(candidate.distances.iter().skip(
                        candidate2.begin_coord.1 - candidate.begin_coord.1,
                    ))
                    .enumerate()
            {
                if d_current > d_next {
                    next_begin = Some((
                        candidate2.begin_coord.0,
                        candidate2.begin_coord.1 + i,
                    ));
                    break;
                }
            }

            previous_begin = Some(next_begin.unwrap_or(end));
            current = next;
        }

        result
    }

    fn add_point(
        &mut self,
        coord: SegmentCoordinate,
        point: &Point,
    ) -> Option<SegmentCoordinate> {
        for candidate in self.candidates.values_mut().flatten() {
            candidate.add_point(coord, point);
        }
        for (id, piste) in self
            .ski_area
            .pistes
            .iter()
            .filter(|(id, _)| self.bounding_rects[*id].intersects(point))
        {
            let entry = self.candidates.entry(id.clone());
            if let Entry::Occupied(e) = &entry {
                if !e.get().iter().all(|c| c.is_finished()) {
                    continue;
                }
            }

            let (is_ok, d) = check_distance(piste, point);
            eprintln!("+ {coord:?} {}: {is_ok:?}, {d}", piste.metadata.name);
            if is_ok {
                entry.or_default().push(Candidate::new(piste, coord, d));
            }
        }

        match (self.first_empty, self.candidates.is_empty()) {
            (None, false) => (),
            (None, true) => self.first_empty = Some(coord),
            (Some(c), false) => {
                self.first_empty = None;
                return Some(c);
            }
            (Some(_), true) => (),
        };
        None
    }

    fn is_all_finished(&self) -> bool {
        self.candidates.values().flatten().all(|c| c.is_finished())
    }
}

pub fn find_pistes(
    cancel: &CancellationToken,
    ski_area: &SkiArea,
    segments: &Segments,
    input: Vec<(MoveType, SegmentCoordinate)>,
) -> Result<Vec<(Moving, SegmentCoordinate)>> {
    let mut result = Vec::new();
    result.reserve(input.len());

    let mut candidates = Candidates::new(ski_area);

    for i in 0..input.len() {
        let (move_type, begin_coord) = input[i];

        let end_coord = input
            .get(i + 1)
            .map(|m| m.1)
            .unwrap_or_else(|| segments.end_coord());
        for (coord, point) in segments.iter_between(begin_coord, end_coord) {
            cancel.check()?;
            if coord.1 == 0 || candidates.is_all_finished() {
                result.append(&mut candidates.commit(move_type, coord));
            }

            if let Some(c) = candidates.add_point(coord, &point.point()) {
                result.push((
                    Moving {
                        move_type,
                        piste_id: String::new(),
                    },
                    c,
                ));
            }
        }

        result.append(&mut candidates.commit(move_type, end_coord));
    }

    Ok(result)
}
