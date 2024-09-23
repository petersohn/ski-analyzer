use super::{format_time_option, to_odt};
use crate::config::get_config;

use gpx::{Gpx, Time, Waypoint};

use std::mem;

pub type Segment<'g> = Vec<&'g Waypoint>;
pub type Segments<'g> = Vec<Segment<'g>>;

const PRECISION_LIMIT: f64 = 10.0;

pub fn get_segments<'g>(gpx: &'g Gpx) -> Segments<'g> {
    let mut result = Vec::new();
    let config = get_config();

    struct BadPrecisionDebug {
        begin: Option<Time>,
        end: Option<Time>,
        min_precision: f64,
        max_precision: f64,
    }

    for track in &gpx.tracks {
        for segment in &track.segments {
            let mut add = |current: &mut Vec<&'g Waypoint>| {
                if !current.is_empty() {
                    result.push(mem::take(current));
                }
            };
            let mut current = Vec::new();

            let mut bad_precision_debug: Option<BadPrecisionDebug> = None;

            for waypoint in &segment.points {
                let precision = match waypoint.hdop {
                    Some(p) => p,
                    None => 0.0,
                };
                if precision > PRECISION_LIMIT {
                    add(&mut current);
                    if config.is_vv() {
                        if let Some(bpd) = bad_precision_debug.as_mut() {
                            bpd.min_precision =
                                bpd.min_precision.min(precision);
                            bpd.max_precision =
                                bpd.max_precision.max(precision);
                            bpd.end = waypoint.time;
                        } else {
                            bad_precision_debug = Some(BadPrecisionDebug {
                                begin: waypoint.time,
                                end: waypoint.time,
                                min_precision: precision,
                                max_precision: precision,
                            });
                        }
                    }
                } else {
                    if config.is_vv() {
                        if let Some(bpd) = bad_precision_debug.as_ref() {
                            eprintln!(
                                "Bad precision between {} and {}: {} - {} m",
                                format_time_option(to_odt(bpd.begin)),
                                format_time_option(to_odt(bpd.end)),
                                bpd.min_precision,
                                bpd.max_precision
                            );
                        }
                        bad_precision_debug = None;
                    }
                    current.push(waypoint);
                }
            }
            add(&mut current);
        }
    }

    result
}