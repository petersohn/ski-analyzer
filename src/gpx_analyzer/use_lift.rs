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
    current_ratio: f64,
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
            LiftCandidate {
                data: UseLift {
                    lift,
                    begin_time: point.time.map(|t| t.into()),
                    end_time: None,
                    begin_station: get_station(lift, &p),
                    end_station: None,
                    is_reverse: false,
                },
                route: vec![vec![point]],
                previous_cutoff,
                result: LiftResult::NotFinished,
                lift_length,
                avg_distance: Avg::new(),
                current_ratio: distance / lift_length,
                direction_known: false,
            }
        })
    }

    fn find(
        ski_area: &'s SkiArea,
        previous_cutoff: (usize, usize),
        point: &'g Waypoint,
    ) -> Vec<LiftCandidate<'s, 'g>> {
        ski_area
            .lifts
            .iter()
            .filter(|l| l.line.bounding_rect().intersects(&point.point()))
            .filter_map(|l| LiftCandidate::new(l, previous_cutoff, point))
            .collect()
    }

    fn continue_lift(&mut self, point: &'g Waypoint) -> LiftResult {
        if self.result != LiftResult::NotFinished {
            return self.result;
        }
        LiftResult::NotFinished
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
