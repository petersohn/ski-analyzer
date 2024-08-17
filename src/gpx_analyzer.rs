use gpx::{Gpx, Time, Waypoint};
use time::OffsetDateTime;

use crate::config::get_config;
use crate::ski_area::{Lift, SkiArea};

#[derive(Debug)]
pub enum LiftEnd {
    Unknown,
    EndStation,
    Midstation(usize),
}

#[derive(Debug)]
pub struct UseLift<'s> {
    lift: &'s Lift,
    begin_time: OffsetDateTime,
    end_time: OffsetDateTime,
    begin_station: Option<LiftEnd>,
    end_station: Option<LiftEnd>,
    is_reverse: bool,
}

#[derive(Debug)]
pub enum ActivityType<'s> {
    UseLift(UseLift<'s>),
}

const PRECISION_LIMIT: f64 = 10.0;

fn get_segments<'g>(gpx: &'g Gpx) -> Vec<Vec<&'g Waypoint>> {
    let mut result = Vec::new();
    let config = get_config();

    struct BadPrecisionDebug {}

    for track in &gpx.tracks {
        for segment in &track.segments {
            let mut add = |current: &mut Vec<&'g Waypoint>| {
                let is_new = !current.is_empty();
                if is_new {
                    let mut to_add = Vec::new();
                    to_add.append(current);
                    result.push(to_add);
                }
                is_new
            };
            let mut current = Vec::new();

            let mut bad_precision_begin: Option<Time> = None;
            let mut bad_precision_end: Option<Time> = None;
            let mut min_precision: f64 = 0.0;
            let mut max_precision: f64 = 0.0;

            for waypoint in &segment.points {
                let precision = match waypoint.hdop {
                    Some(p) => p,
                    None => 0.0,
                };
                if precision > PRECISION_LIMIT {
                    let is_new = add(&mut current);
                    if config.is_vv() {
                        if is_new {
                            bad_precision_begin = waypoint.time;
                            min_precision = precision;
                            max_precision = precision;
                        } else {
                            min_precision = min_precision.min(precision);
                            max_precision = max_precision.max(precision);
                        }
                        bad_precision_end = waypoint.time;
                    }
                } else {
                    if config.is_vv() {
                        if let Some(begin) = bad_precision_begin {
                            if let Some(end) = bad_precision_end {
                                eprintln!(
                                    "Bad precision between {} and {}: {} - {} m",
                                    begin.format().unwrap(),
                                    end.format().unwrap(),
                                    min_precision,
                                    max_precision);
                                bad_precision_end = None;
                            }
                            bad_precision_begin = None;
                        }
                    }
                    current.push(waypoint);
                }
            }
            add(&mut current);
        }
    }

    result
}

pub fn analyze_route<'s>(
    ski_area: &'s SkiArea,
    gpx: &Gpx,
) -> Vec<ActivityType<'s>> {
    let mut result = Vec::new();

    let segments = get_segments(&gpx);
    println!("{:#?}", segments);

    result
}
