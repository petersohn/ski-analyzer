use super::{Activity, ActivityType, Segment, Segments};
use crate::ski_area::{Lift, SkiArea};
use crate::utils::collection::Avg;

use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::take;

use geo::{Distance, Haversine, Intersects, Length, Point, Rect};
use gpx::Waypoint;
use serde::{Deserialize, Serialize};

const MIN_DISTANCE: f64 = 10.0;

pub type LiftEnd = Option<usize>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UseLift {
    pub lift_id: String,
    pub begin_station: LiftEnd,
    pub end_station: LiftEnd,
    pub is_reverse: bool,
}

fn get_station(lift: &Lift, p: Point) -> LiftEnd {
    lift.stations
        .iter()
        .enumerate()
        .map(|(i, m)| (i, Haversine::distance(m.point, p)))
        .filter(|(_, m)| *m < MIN_DISTANCE)
        .min_by(|(_, d1), (_, d2)| d1.total_cmp(d2))
        .map(|(i, _)| i)
}

struct LiftDistance {
    from_begin: f64,
    from_line: f64,
}

impl LiftDistance {
    fn get(lift: &Lift, p: Point) -> Option<Self> {
        let distance = lift.get_closest_point(p)?;
        if distance.distance > MIN_DISTANCE {
            return None;
        }
        Some(LiftDistance {
            from_begin: lift
                .line
                .item
                .lines()
                .take(distance.line_id)
                .fold(0.0, |acc, l| acc + l.length::<Haversine>())
                + Haversine::distance(
                    distance.point,
                    distance.line.start.into(),
                ),
            from_line: distance.distance,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LiftResult {
    NotFinished,
    Finished,
    Failure,
}

type SegmentCoordinate = (usize, usize);

#[derive(Debug)]
struct LiftCandidate<'s> {
    lift: &'s Lift,
    data: UseLift,
    result: LiftResult,
    lift_length: f64,
    possible_begins: Vec<SegmentCoordinate>,
    possible_ends: Vec<SegmentCoordinate>,
    avg_distance: Avg<f64>,
    distance_from_begin: f64,
    direction_known: bool,
}

impl<'s> LiftCandidate<'s> {
    fn create(
        unique_id: String,
        lift: &'s Lift,
        coordinate: SegmentCoordinate,
        point: &Waypoint,
    ) -> Option<Self> {
        let p = point.point();
        LiftDistance::get(lift, p).and_then(|distance| {
            let lift_length = lift.line.item.length::<Haversine>();
            let station = get_station(lift, p);
            if station.is_none() && coordinate.1 != 0 {
                return None;
            }
            let mut avg_distance = Avg::new();
            avg_distance.add(distance.from_line);
            Some(LiftCandidate {
                lift,
                data: UseLift {
                    lift_id: unique_id,
                    begin_station: station,
                    end_station: None,
                    is_reverse: false,
                },
                possible_begins: vec![coordinate],
                possible_ends: vec![],
                result: LiftResult::NotFinished,
                lift_length,
                avg_distance,
                distance_from_begin: distance.from_begin,
                direction_known: false,
            })
        })
    }

    fn find<It>(
        bouning_rects: &HashMap<String, Rect>,
        it: It,
        coordinate: SegmentCoordinate,
        point: &Waypoint,
    ) -> Vec<LiftCandidate<'s>>
    where
        It: Iterator<Item = (&'s String, &'s Lift)>,
    {
        it.filter(|(id, _)| bouning_rects[*id].intersects(&point.point()))
            .filter_map(|(id, l)| {
                LiftCandidate::create(id.clone(), l, coordinate, point)
            })
            .collect()
    }

    fn leave(&mut self, coordinate: SegmentCoordinate) -> LiftResult {
        if coordinate.1 == 0 // We might have lost some data
                    // You fell out of a draglift
                    || (self.lift.can_disembark
                        && !self.possible_ends.is_empty())
                    || self.data.end_station.is_some()
        {
            self.transition(LiftResult::Finished)
        } else {
            self.transition(LiftResult::Failure)
        }
    }

    fn add_point(
        &mut self,
        point: &Waypoint,
        coordinate: SegmentCoordinate,
    ) -> LiftResult {
        if self.result != LiftResult::NotFinished {
            panic!("Already finished");
        }
        let p = point.point();
        let distance = match LiftDistance::get(self.lift, p) {
            Some(d) => d,
            None => return self.leave(coordinate),
        };
        if (distance.from_begin - self.distance_from_begin).abs() > MIN_DISTANCE
        {
            let reverse = distance.from_begin < self.distance_from_begin;
            if !self.direction_known {
                if reverse && !self.lift.can_go_reverse {
                    return self.transition(LiftResult::Failure);
                }
                self.direction_known = true;
                self.data.is_reverse = reverse;
            } else if reverse != self.data.is_reverse {
                return self.leave(coordinate);
            }
            self.distance_from_begin = distance.from_begin;
        }
        self.avg_distance.add(distance.from_line);
        let station = get_station(self.lift, p);
        match station {
            Some(s) => {
                if self.data.begin_station == Some(s) {
                    self.possible_begins.push(coordinate);
                } else {
                    self.data.end_station = station;
                    self.possible_ends.push(coordinate);
                }
            }
            None => {
                self.data.end_station = None;
                self.possible_ends.clear();
                if self.lift.can_disembark {
                    self.possible_ends.push(coordinate);
                }
            }
        }
        LiftResult::NotFinished
    }

    fn transition(&mut self, result: LiftResult) -> LiftResult {
        self.result = result;
        result
    }

    fn commit(self) -> UseLift {
        self.data
    }

    fn found_station_count(&self) -> u32 {
        self.data.begin_station.is_some() as u32
            + self.data.end_station.is_some() as u32
    }

    fn can_go_after(&self, other: &LiftCandidate) -> bool {
        self.possible_begins.last().unwrap()
            >= other.possible_ends.first().unwrap()
    }
}

fn group_lift_candidates<'s>(
    mut candidates: Vec<LiftCandidate<'s>>,
) -> Vec<Vec<LiftCandidate<'s>>> {
    let mut result = Vec::new();
    while let Some(candidate) = candidates.pop() {
        let (mut group, rest): (
            Vec<LiftCandidate<'s>>,
            Vec<LiftCandidate<'s>>,
        ) = candidates.into_iter().partition(|c| {
            c.found_station_count() == candidate.found_station_count()
                && (c.lift_length - candidate.lift_length).abs() < MIN_DISTANCE
        });
        group.push(candidate);
        group.sort_by(|lhs, rhs| {
            lhs.avg_distance
                .get()
                .partial_cmp(&rhs.avg_distance.get())
                .unwrap()
        });
        result.push(group);
        candidates = rest;
    }
    result
}

fn commit_lift_candidates<'s>(
    candidates: Vec<LiftCandidate<'s>>,
) -> Vec<(ActivityType, SegmentCoordinate)> {
    let mut groups = group_lift_candidates(candidates);

    groups.sort_by(|lhs, rhs| {
        (lhs[0].found_station_count(), -lhs[0].lift_length)
            .partial_cmp(&(rhs[0].found_station_count(), -rhs[0].lift_length))
            .unwrap()
    });

    let mut candidates2 = Vec::new();
    for g in groups.into_iter() {
        for c in g.into_iter() {
            if candidates2
                .iter()
                .all(|c2| c.can_go_after(&c2) || c2.can_go_after(&c))
            {
                candidates2.push(c);
            }
        }
    }

    candidates2.sort_by(|lhs, rhs| {
        rhs.possible_begins[0].cmp(&lhs.possible_begins[0])
    });

    let mut result = Vec::new();
    let mut current = candidates2.pop().unwrap();
    let mut coord = current.possible_begins[0];
    while let Some(next) = candidates2.pop() {
        let current_end = *current.possible_ends.last().unwrap();
        let next_begin = *next.possible_begins.first().unwrap();
        result.push((ActivityType::UseLift(current.commit()), coord));
        coord = if current_end < next_begin {
            result.push((ActivityType::default(), current_end));
            next_begin
        } else {
            *next
                .possible_begins
                .iter()
                .rfind(|c| **c <= current_end)
                .unwrap()
        };
        current = next;
    }
    result.push((ActivityType::UseLift(current.commit()), coord));
    result
}

fn split_route(route: &mut Segments, coord: SegmentCoordinate) -> Segments {
    if coord.1 == 0 {
        route.drain(coord.0..).collect()
    } else {
        let first_segment: Segment = route[coord.0].drain(coord.1..).collect();
        if coord.0 == route.len() - 1 {
            vec![first_segment]
        } else {
            [first_segment]
                .into_iter()
                .chain(route.drain(coord.0 + 1..))
                .collect()
        }
    }
}

pub fn find_lift_usage<'s>(
    ski_area: &'s SkiArea,
    segments: Segments,
) -> Vec<Activity> {
    let mut result = Vec::new();

    let bounding_rects: HashMap<String, Rect> = ski_area
        .lifts
        .iter()
        .map(|(id, l)| (id.clone(), l.line.expanded_rect(MIN_DISTANCE)))
        .collect();

    type Candidates<'s> = Vec<LiftCandidate<'s>>;
    let mut current_route: Segments = Vec::new();
    let mut candidates: Candidates = Vec::new();
    let mut finished_candidates: Candidates = Vec::new();

    for segment in segments {
        let mut route_segment: Segment = Vec::new();
        for point in segment {
            let mut coordinate = (current_route.len(), route_segment.len());
            let (mut finished, unfinished): (Candidates, Candidates) =
                candidates
                    .into_iter()
                    .filter_map(|mut l| match l.add_point(&point, coordinate) {
                        LiftResult::Failure => None,
                        _ => Some(l),
                    })
                    .partition(|l| l.result == LiftResult::Finished);
            candidates = unfinished;
            finished_candidates.append(&mut finished);

            if candidates.is_empty() && !finished_candidates.is_empty() {
                if !route_segment.is_empty() {
                    current_route.push(take(&mut route_segment));
                }
                let mut to_add: Vec<Activity> = Vec::new();
                for (type_, coord) in
                    commit_lift_candidates(take(&mut finished_candidates))
                        .into_iter()
                        .rev()
                {
                    let route = split_route(&mut current_route, coord);
                    to_add.push(Activity::new(type_, route));
                }
                if !current_route.is_empty() {
                    to_add.push(Activity::new(
                        ActivityType::default(),
                        take(&mut current_route),
                    ));
                }
                result.reserve(to_add.len());
                to_add.into_iter().rev().for_each(|r| result.push(r));
                coordinate = (current_route.len(), route_segment.len());
            }

            let mut new_candidates = LiftCandidate::find(
                &bounding_rects,
                ski_area.lifts.iter().filter(|l| {
                    candidates
                        .iter()
                        .chain(finished_candidates.iter())
                        .find(|c| {
                            (*&l.1 as *const Lift) == (c.lift as *const Lift)
                        })
                        .is_none()
                }),
                coordinate,
                &point,
            );
            candidates.append(&mut new_candidates);

            route_segment.push(point);
        }
        current_route.push(route_segment);
    }

    if !current_route.is_empty() {
        result.push(Activity::new(ActivityType::default(), current_route));
    }

    result
}
