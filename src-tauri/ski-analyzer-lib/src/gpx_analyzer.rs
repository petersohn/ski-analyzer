use geo::{Distance, Haversine, Length, Line};
use gpx::{Gpx, Time, Waypoint};
use gpx_parser::parse_gpx;
use serde::{Deserialize, Serialize};
use std::mem::take;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::error::Result;
use crate::ski_area::SkiArea;
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::cancel::CancellationToken;
use crate::utils::option_time_ser;
use use_lift::find_lift_usage;

mod gpx_parser;
mod moving;
mod segments;
mod use_lift;
mod waypoint_ser;

#[cfg(test)]
mod gpx_parser_test;
#[cfg(test)]
mod segments_test;
#[cfg(test)]
mod test_util;
#[cfg(test)]
mod use_lift_test;

pub use moving::Moving;
pub use segments::{Segment, SegmentCoordinate, Segments};
pub use use_lift::{LiftEnd, UseLift};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ActivityType {
    Unknown(()),
    UseLift(UseLift),
    EnterLift(String),
    ExitLift(String),
    Moving(Moving),
}

impl Default for ActivityType {
    fn default() -> Self {
        ActivityType::Unknown(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Activity {
    #[serde(rename = "type")]
    pub type_: ActivityType,
    pub route: Segments,
    #[serde(with = "option_time_ser")]
    pub begin_time: Option<OffsetDateTime>,
    #[serde(with = "option_time_ser")]
    pub end_time: Option<OffsetDateTime>,
    pub length: f64,
}

impl Activity {
    fn new(type_: ActivityType, route: Segments) -> Self {
        let begin_time = route
            .0
            .first()
            .map(|s| s.first())
            .flatten()
            .map(|wp| wp.time.map(|t| t.into()))
            .flatten();
        let end_time = route
            .0
            .last()
            .map(|s| s.last())
            .flatten()
            .map(|wp| wp.time.map(|t| t.into()))
            .flatten();
        let length = route
            .0
            .iter()
            .map(|s| {
                s.windows(2).map(|wps| {
                    Line::new(wps[0].point(), wps[1].point())
                        .length::<Haversine>()
                })
            })
            .flatten()
            .sum();

        Activity {
            type_,
            route,
            begin_time,
            end_time,
            length,
        }
    }
}

fn to_odt(time: Option<Time>) -> Option<OffsetDateTime> {
    time.map(|t| t.into())
}

fn format_time_option(time: Option<OffsetDateTime>) -> String {
    match time {
        Some(t) => format!("{}", t.format(&Iso8601::DEFAULT).unwrap()),
        None => "unknown".to_string(),
    }
}

pub type AnalyzedRoute = BoundedGeometry<Vec<Activity>>;

pub fn analyze_route(
    cancel: &CancellationToken,
    ski_area: &SkiArea,
    gpx: Gpx,
) -> Result<AnalyzedRoute> {
    let mut segments = parse_gpx(gpx)?;
    let result = find_lift_usage(cancel, ski_area, take(&mut segments.item))?;
    Ok(BoundedGeometry {
        item: result,
        bounding_rect: segments.bounding_rect,
    })
}

fn get_speed_inner(
    wp1: &Waypoint,
    wp2: &Waypoint,
    distance: f64,
) -> Option<f64> {
    let t1 = OffsetDateTime::from(wp1.time?);
    let t2 = OffsetDateTime::from(wp2.time?);
    if t1 == t2 {
        None
    } else {
        let t = (t2 - t1).as_seconds_f64();
        Some(distance / t)
    }
}

fn get_speed(wp1: &Waypoint, wp2: &Waypoint) -> Option<f64> {
    let p1 = wp1.point();
    let p2 = wp2.point();
    if p1 == p2 {
        return Some(0.0);
    }
    let distance = Haversine::distance(wp1.point(), wp2.point());
    get_speed_inner(wp1, wp2, distance)
}

fn get_inclination_inner(
    wp1: &Waypoint,
    wp2: &Waypoint,
    distance: f64,
) -> Option<f64> {
    let e1 = wp1.elevation?;
    let e2 = wp2.elevation?;
    Some((e2 - e1) / distance)
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct DerivedData {
    pub speed: Option<f64>,
    pub inclination: Option<f64>,
}

impl DerivedData {
    fn calculate_inner(wp1: &Waypoint, wp2: &Waypoint) -> (Self, f64) {
        let p1 = wp1.point();
        let p2 = wp2.point();
        // Optimization: we don't have to calculate distance if the two points are the
        // same.
        if p1 == p2 {
            return (
                Self {
                    speed: Some(0.0),
                    inclination: None,
                },
                0.0,
            );
        }
        let distance = Haversine::distance(wp1.point(), wp2.point());
        (
            Self {
                speed: get_speed_inner(wp1, wp2, distance),
                inclination: get_inclination_inner(wp1, wp2, distance),
            },
            distance,
        )
    }

    pub fn calculate(wp1: &Waypoint, wp2: &Waypoint) -> Self {
        DerivedData::calculate_inner(wp1, wp2).0
    }
}
