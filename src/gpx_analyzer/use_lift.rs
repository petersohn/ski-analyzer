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

fn continue_lift<'s>(lift: &mut UseLift<'s>, point: &Waypoint) -> LiftResult {
    LiftResult::NotFinished
}

pub fn find_lift_usage<'s, 'g>(
    ski_area: &'s SkiArea,
    segments: &Segments<'g>,
) -> Vec<Activity<'s, 'g>> {
    let mut result = Vec::new();

    let mut current: Activity<'s, 'g> = Activity {
        type_: ActivityType::Unknown,
        route: Vec::new(),
    };

    let config = get_config();

    for segment in segments {
        let mut first_point = true;
        for point in segment {
            if let ActivityType::UseLift(lift) = &mut current.type_ {
                match continue_lift(lift, point) {
                    LiftResult::NotFinished => (),
                    LiftResult::Finished => {}
                    LiftResult::Failure => {
                        if config.is_vv() {
                            eprintln!("Failed to parse lift after {} points, begin={} end={}",
                                current.route.iter().map(|s| s.len()).fold(0, |a, b| a + b),
                                format_time_option(lift.begin_time),
                                format_time_option(to_odt(point.time)));
                        }
                    }
                }
            }
            first_point = false;
        }
    }

    result
}
