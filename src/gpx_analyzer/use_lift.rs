use super::segments::Segments;
use super::{format_time_option, to_odt};
use super::{Activity, ActivityType, LiftEnd, UseLift};
use crate::config::get_config;
use crate::ski_area::SkiArea;

use geo::{HaversineDistance, Point};
use gpx::Waypoint;

const MIN_DISTANCE: f64 = 10.0;

fn is_near<P1, P2>(p1: &P1, p2: &P2) -> bool
where
    P1: HaversineDistance<f64, P2>,
{
    p1.haversine_distance(p2) < MIN_DISTANCE
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

fn handle_failed_lift<'s, 'g>(
    previous: Option<&mut Activity<'s, 'g>>,
    current: Activity<'s, 'g>,
    starts_with_new_segment: bool,
) {
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
                    LiftResult::Finished => {}
                    LiftResult::Failure => {
                        if config.is_vv() {
                            eprintln!("Failed to parse lift after {} points, begin={} end={}",
                                current.route.iter().map(|s| s.len()).fold(0, |a, b| a + b),
                                format_time_option(lift.begin_time),
                                format_time_option(to_odt(point.time)));
                        }
                        current.type_ = ActivityType::Unknown;
                        let previous = result.last_mut();
                        match previous {
                            Some(Activity {
                                type_: ActivityType::Unknown,
                                ref mut route,
                            }) => {
                                if !starts_with_new_segment {
                                    route
                                        .last_mut()
                                        .unwrap()
                                        .append(&mut current.route.remove(0));
                                }
                                route.append(&mut current.route);
                            }
                            _ => result.push(std::mem::take(&mut current)),
                        };
                    }
                }
            }
            first_point = false;
        }
    }

    result
}
