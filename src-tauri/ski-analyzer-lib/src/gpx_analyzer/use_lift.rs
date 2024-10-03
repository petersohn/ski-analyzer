use super::{serialize_unique_id, Activity, ActivityType, Segment, Segments};
use crate::collection::Avg;
use crate::ski_area::{Lift, SkiArea};

use std::fmt::Debug;
use std::mem::take;
use std::result::Result as StdResult;

use geo::{
    BoundingRect, Closest, HaversineClosestPoint, HaversineDistance,
    HaversineLength, Intersects, Point,
};
use gpx::Waypoint;
use serde::{Serialize, Serializer};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

const MIN_DISTANCE: f64 = 10.0;

pub type LiftEnd = Option<usize>;

fn serialize_time<S>(
    time: &Option<OffsetDateTime>,
    serializer: S,
) -> StdResult<S::Ok, S::Error>
where
    S: Serializer,
{
    match time {
        None => serializer.serialize_unit(),
        Some(t) => {
            let s = t
                .format(&Iso8601::DEFAULT)
                .map_err(|e| serde::ser::Error::custom(e))?;
            serializer.serialize_str(&s)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UseLift<'s> {
    #[serde(serialize_with = "serialize_unique_id")]
    pub lift: &'s Lift,
    #[serde(serialize_with = "serialize_time")]
    pub begin_time: Option<OffsetDateTime>,
    #[serde(serialize_with = "serialize_time")]
    pub end_time: Option<OffsetDateTime>,
    pub begin_station: LiftEnd,
    pub end_station: LiftEnd,
    pub is_reverse: bool,
}

fn get_station(lift: &Lift, p: &Point) -> LiftEnd {
    lift.stations
        .iter()
        .enumerate()
        .map(|(i, m)| (i, m.point.haversine_distance(p)))
        .filter(|(_, m)| *m < MIN_DISTANCE)
        .min_by(|(_, d1), (_, d2)| d1.total_cmp(d2))
        .map(|(i, _)| i)
}

struct Distance {
    from_begin: f64,
    from_line: f64,
}

impl Distance {
    fn get(lift: &Lift, p: &Point) -> Option<Self> {
        let (segment, line, p2, from_line) = lift
            .line
            .item
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let p2 = match line.haversine_closest_point(p) {
                    Closest::Intersection(p2) => p2,
                    Closest::SinglePoint(p2) => p2,
                    Closest::Indeterminate => {
                        panic!(
                            "Cannot determine distance between {:?} and {:?}",
                            p, line
                        );
                    }
                };
                (i, line, p2, p.haversine_distance(&p2))
            })
            .min_by(|(_, _, _, d1), (_, _, _, d2)| d1.total_cmp(d2))?;
        if from_line > MIN_DISTANCE {
            return None;
        }
        Some(Distance {
            from_begin: lift
                .line
                .item
                .lines()
                .take(segment)
                .fold(0.0, |acc, l| acc + l.haversine_length())
                + p2.haversine_distance(&line.start.into()),
            from_line,
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
    data: UseLift<'s>,
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
        lift: &'s Lift,
        coordinate: SegmentCoordinate,
        point: &Waypoint,
    ) -> Option<Self> {
        let p = point.point();
        Distance::get(lift, &p).and_then(|distance| {
            let lift_length = lift.line.item.haversine_length();
            let station = get_station(lift, &p);
            if station.is_none() && coordinate.1 != 0 {
                return None;
            }
            let mut avg_distance = Avg::new();
            avg_distance.add(distance.from_line);
            Some(LiftCandidate {
                data: UseLift {
                    lift,
                    begin_time: point.time.map(|t| t.into()),
                    end_time: None,
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
        it: It,
        coordinate: SegmentCoordinate,
        point: &Waypoint,
    ) -> Vec<LiftCandidate<'s>>
    where
        It: Iterator<Item = &'s Lift>,
    {
        it.filter(|l| l.line.bounding_rect().intersects(&point.point()))
            .filter_map(|l| LiftCandidate::create(l, coordinate, point))
            .collect()
    }

    fn leave(&mut self, coordinate: SegmentCoordinate) -> LiftResult {
        if coordinate.1 == 0 // We might have lost some data
                    // You fell out of a draglift
                    || (self.data.lift.can_disembark
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
        let distance = match Distance::get(self.data.lift, &p) {
            Some(d) => d,
            None => return self.leave(coordinate),
        };
        if (distance.from_begin - self.distance_from_begin).abs() > MIN_DISTANCE
        {
            let reverse = distance.from_begin < self.distance_from_begin;
            if !self.direction_known {
                if reverse && !self.data.lift.can_go_reverse {
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
        let station = get_station(self.data.lift, &p);
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
                if self.data.lift.can_disembark {
                    self.possible_ends.push(coordinate);
                }
            }
        }
        self.data.end_time = point.time.map(|t| t.into());
        LiftResult::NotFinished
    }

    fn transition(&mut self, result: LiftResult) -> LiftResult {
        self.result = result;
        result
    }

    fn commit(self) -> UseLift<'s> {
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
) -> Vec<(ActivityType<'s>, SegmentCoordinate)> {
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
            result.push((ActivityType::Unknown, current_end));
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

fn split_route<'g>(
    route: &mut Segments<'g>,
    coord: SegmentCoordinate,
) -> Segments<'g> {
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

pub fn find_lift_usage<'s, 'g>(
    ski_area: &'s SkiArea,
    segments: &Segments<'g>,
) -> Vec<Activity<'s, 'g>> {
    let mut result = Vec::new();

    type Candidates<'s> = Vec<LiftCandidate<'s>>;
    let mut current_route: Segments<'g> = Vec::new();
    let mut candidates: Candidates = Vec::new();
    let mut finished_candidates: Candidates = Vec::new();

    for segment in segments {
        let mut route_segment: Segment = Vec::new();
        for point in segment {
            let mut coordinate = (current_route.len(), route_segment.len());
            let (mut finished, unfinished): (Candidates, Candidates) =
                candidates
                    .into_iter()
                    .filter_map(|mut l| match l.add_point(point, coordinate) {
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
                let mut to_add: Vec<Activity<'s, 'g>> = Vec::new();
                for (type_, coord) in
                    commit_lift_candidates(take(&mut finished_candidates))
                        .into_iter()
                        .rev()
                {
                    let route = split_route(&mut current_route, coord);
                    to_add.push(Activity { type_, route });
                }
                if !current_route.is_empty() {
                    to_add.push(Activity {
                        type_: ActivityType::Unknown,
                        route: take(&mut current_route),
                    });
                }
                result.reserve(to_add.len());
                to_add.into_iter().rev().for_each(|r| result.push(r));
                coordinate = (current_route.len(), route_segment.len());
            }

            let mut new_candidates = LiftCandidate::find(
                ski_area.lifts.iter().filter(|l| {
                    candidates
                        .iter()
                        .chain(finished_candidates.iter())
                        .find(|c| {
                            (*l as *const Lift) == (c.data.lift as *const Lift)
                        })
                        .is_none()
                }),
                coordinate,
                point,
            );
            candidates.append(&mut new_candidates);

            route_segment.push(point);
        }
        current_route.push(route_segment);
    }

    if !current_route.is_empty() {
        result.push(Activity {
            type_: ActivityType::Unknown,
            route: current_route,
        });
    }

    result
}
