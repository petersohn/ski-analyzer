use super::segments::Segments;
use super::{format_time_option, to_odt};
use super::{Activity, ActivityType, LiftEnd, UseLift};
use crate::collection::Avg;
use crate::config::get_config;
use crate::ski_area::{Lift, SkiArea};

use std::fmt::Debug;

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

struct LiftCandidate<'s, 'g> {
    data: UseLift<'s>,
    route: Segments<'g>,
    previous_cutoff: (usize, usize),
    result: LiftResult,
    lift_length: f64,
    avg_distance: Avg<f64>,
    distance_from_begin: f64,
    direction_known: bool,
}

impl<'s, 'g> LiftCandidate<'s, 'g> {
    fn new(
        lift: &'s Lift,
        previous_cutoff: (usize, usize),
        point: &'g Waypoint,
    ) -> Option<Self> {
        let p = point.point();
        get_distance_from_begin(lift, &p).map(|distance| {
            let lift_length = lift.line.item.haversine_length();
            let station = get_station(lift, &p);
            LiftCandidate {
                data: UseLift {
                    lift,
                    begin_time: point.time.map(|t| t.into()),
                    end_time: None,
                    begin_station: station,
                    end_station: station,
                    is_reverse: false,
                },
                route: vec![vec![point]],
                previous_cutoff,
                result: LiftResult::NotFinished,
                lift_length,
                avg_distance: Avg::new(),
                distance_from_begin: distance,
                direction_known: false,
            }
        })
    }

    fn find<It>(
        it: It,
        previous_cutoff: (usize, usize),
        point: &'g Waypoint,
    ) -> Vec<LiftCandidate<'s, 'g>>
    where
        It: Iterator<Item = &'s Lift>,
    {
        it.filter(|l| l.line.bounding_rect().intersects(&point.point()))
            .filter_map(|l| LiftCandidate::new(l, previous_cutoff, point))
            .collect()
    }

    fn add_point(
        &mut self,
        point: &'g Waypoint,
        is_new_segment: bool,
    ) -> LiftResult {
        if self.result != LiftResult::NotFinished {
            return self.result;
        }
        let p = point.point();
        let distance = match get_distance_from_begin(self.data.lift, &p) {
            Some(d) => d,
            None => {
                if is_new_segment // We might have lost some data
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
        self.data.end_station = get_station(self.data.lift, &p);
        self.data.end_time = point.time.map(|t| t.into());
        LiftResult::NotFinished
    }

    fn transition(&mut self, result: LiftResult) -> LiftResult {
        self.result = result;
        result
    }
}

fn continue_lift<'s>(
    lift: &mut UseLift<'s>,
    point: &Waypoint,
    first_point: bool,
) -> LiftResult {
    LiftResult::NotFinished
}

pub fn find_lift_usage<'s, 'g>(
    ski_area: &'s SkiArea,
    segments: &Segments<'g>,
) -> Vec<Activity<'s, 'g>> {
    let mut result = Vec::new();

    let mut current: Activity<'s, 'g> = Activity::default();
    let mut starts_with_new_segment = false;

    let config = get_config();

    for segment in segments {
        let mut first_point = true;
        for point in segment {
            if let ActivityType::UseLift(lift) = &mut current.type_ {
                match continue_lift(lift, point, first_point) {
                    LiftResult::NotFinished => (),
                    LiftResult::Finished => {
                        result.push(std::mem::take(&mut current))
                    }
                    LiftResult::Failure => {
                        if config.is_vv() {
                            eprintln!("Failed to parse lift after {} points, begin={} end={}",
                                current.route.iter().map(|s| s.len()).fold(0, |a, b| a + b),
                                format_time_option(lift.begin_time),
                                format_time_option(to_odt(point.time)));
                        }
                        current.type_ = ActivityType::Unknown;
                        if let Some(Activity {
                            type_: ActivityType::Unknown,
                            ref mut route,
                        }) = result.last_mut()
                        {
                            if !starts_with_new_segment {
                                route
                                    .last_mut()
                                    .unwrap()
                                    .append(&mut current.route.remove(0));
                            }
                            route.append(&mut current.route);
                        } else {
                            result.push(std::mem::take(&mut current));
                        }
                    }
                }
            } else {
            }
            first_point = false;
        }
    }

    result
}
