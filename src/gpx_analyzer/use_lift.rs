use super::segments::Segments;
use super::{format_time_option, to_odt};
use super::{Activity, ActivityType, LiftEnd, UseLift};
use crate::config::get_config;
use crate::ski_area::{Lift, SkiArea};

use std::fmt::Debug;

use geo::{
    BoundingRect, Closest, HaversineClosestPoint, HaversineDistance,
    Intersects, Point,
};
use gpx::Waypoint;

const MIN_DISTANCE: f64 = 10.0;

fn is_near(p1: &Point, p2: &Point) -> bool {
    p1.haversine_distance(p2) < MIN_DISTANCE
}

fn is_near_line<T>(t: &T, p: &Point) -> bool
where
    T: HaversineClosestPoint<f64> + Debug,
{
    match t.haversine_closest_point(p) {
        Closest::Intersection(_) => true,
        Closest::SinglePoint(p2) => is_near(p, &p2),
        Closest::Indeterminate => {
            if get_config().is_vv() {
                eprintln!(
                    "Cannot determine distance between {:?} and {:?}",
                    p, t
                );
            }
            false
        }
    }
}

fn get_station(lift: &Lift, p: &Point) -> LiftEnd {
    if is_near(&(*lift.line.item.0.first().unwrap()).into(), p) {
        LiftEnd::BeginStation
    } else if is_near(&(*lift.line.item.0.last().unwrap()).into(), p) {
        LiftEnd::EndStation
    } else {
        lift.midstations
            .iter()
            .enumerate()
            .filter(|(_, m)| is_near(&m.point, p))
            .map(|(i, _)| LiftEnd::Midstation(i))
            .next()
            .unwrap_or(LiftEnd::Unknown)
    }
}

fn find_nearby_lift<'s>(
    ski_area: &'s SkiArea,
    p: &Point,
) -> Vec<(&'s Lift, LiftEnd)> {
    ski_area
        .lifts
        .iter()
        .filter(|l| {
            l.line.bounding_rect().intersects(p)
                && is_near_line(&l.line.item, p)
        })
        .map(|l| (l, get_station(l, p)))
        .collect()
}

enum LiftResult {
    NotFinished,
    Finished,
    Failure,
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
