use geo::{Haversine, Length, Line};
use gpx::{Gpx, Time};
use serde::{Deserialize, Serialize};
use std::mem::take;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::error::Result;
use crate::ski_area::SkiArea;
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::option_time_ser;
use segments::get_segments;
use use_lift::find_lift_usage;

mod segments;
mod segments_ser;
mod use_lift;

#[cfg(test)]
mod segments_test;
#[cfg(test)]
mod test_util;
#[cfg(test)]
mod use_lift_test;

pub use segments::{Segment, Segments};
pub use use_lift::{LiftEnd, UseLift};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ActivityType {
    Unknown(()),
    UseLift(UseLift),
    EnterLift(String),
    ExitLift(String),
}

impl Default for ActivityType {
    fn default() -> Self {
        ActivityType::Unknown(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Activity {
    #[serde(rename = "type")]
    pub type_: ActivityType,
    #[serde(with = "segments_ser")]
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
            .first()
            .map(|s| s.first())
            .flatten()
            .map(|wp| wp.time.map(|t| t.into()))
            .flatten();
        let end_time = route
            .last()
            .map(|s| s.last())
            .flatten()
            .map(|wp| wp.time.map(|t| t.into()))
            .flatten();
        let length = route
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

pub fn analyze_route(ski_area: &SkiArea, gpx: Gpx) -> Result<AnalyzedRoute> {
    let mut segments = get_segments(gpx)?;
    // println!("{:#?}", segments);
    let result = find_lift_usage(ski_area, take(&mut segments.item));
    Ok(BoundedGeometry {
        item: result,
        bounding_rect: segments.bounding_rect,
    })
}
