use super::segments::{Segment, Segments};
use super::{format_time_option, to_odt};
use super::{Activity, ActivityType, LiftEnd, UseLift};
use crate::collection::Avg;
use crate::config::get_config;
use crate::ski_area::{Lift, SkiArea};

use std::fmt::Debug;
use std::mem::take;

use geo::{
    BoundingRect, Closest, HaversineClosestPoint, HaversineDistance,
    HaversineLength, Intersects, Line, Point,
};
use gpx::Waypoint;

const MIN_DISTANCE: f64 = 10.0;

fn is_near(p1: &Point, p2: &Point) -> bool {
    p1.haversine_distance(p2) < MIN_DISTANCE
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

fn get_distance_from_begin(lift: &Lift, p: &Point) -> Option<f64> {
    let (segment, line, p2, distance) = lift
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
    if distance > MIN_DISTANCE {
        return None;
    }
    Some(
        lift.line
            .item
            .lines()
            .take(segment)
            .fold(0.0, |acc, l| acc + l.haversine_length())
            + p2.haversine_distance(&line.start.into()),
    )
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LiftResult {
    NotFinished,
    Finished,
    Failure,
}

type SegmentCoordinate = (usize, usize);

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
    fn new(
        lift: &'s Lift,
        coordinate: SegmentCoordinate,
        point: &Waypoint,
    ) -> Option<Self> {
        let p = point.point();
        get_distance_from_begin(lift, &p).and_then(|distance| {
            let lift_length = lift.line.item.haversine_length();
            let station = get_station(lift, &p);
            if station.is_none() && coordinate.1 != 0 {
                return None;
            }
            Some(LiftCandidate {
                data: UseLift {
                    lift,
                    begin_time: point.time.map(|t| t.into()),
                    end_time: None,
                    begin_station: station,
                    end_station: station,
                    is_reverse: false,
                },
                possible_begins: vec![coordinate],
                possible_ends: vec![],
                result: LiftResult::NotFinished,
                lift_length,
                avg_distance: Avg::new(),
                distance_from_begin: distance,
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
            .filter_map(|l| LiftCandidate::new(l, coordinate, point))
            .collect()
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
        let distance = match get_distance_from_begin(self.data.lift, &p) {
            Some(d) => d,
            None => {
                if coordinate.1 == 0 // We might have lost some data
                    || self.data.lift.can_disembark // You fell out of a draglift
                    || self.data.end_station.is_some()
                {
                    return self.transition(LiftResult::Finished);
                } else {
                    return self.transition(LiftResult::Failure);
                }
            }
        };
        if (distance - self.distance_from_begin).abs() > MIN_DISTANCE {
            let reverse = distance < self.distance_from_begin;
            if !self.direction_known {
                if reverse && !self.data.lift.can_go_reverse {
                    return self.transition(LiftResult::Failure);
                }
                self.direction_known = true;
                self.data.is_reverse = reverse;
            } else if reverse != self.data.is_reverse {
                return self.transition(LiftResult::Failure);
            }
        }
        self.avg_distance.add(distance);
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
}

fn commit_lift_candidates<'s>(
    candidates: Vec<LiftCandidate<'s>>,
) -> Vec<(ActivityType<'s>, SegmentCoordinate)> {
    vec![]
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
            let coordinate = (current_route.len(), route_segment.len());
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
                if !current_route.is_empty() {
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

    result
}
