use geo::{Haversine, Length, Line};
use gpx::{Gpx, Time};
use serde::{Serialize, Serializer};
use std::mem::take;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::error::Result;
use crate::ski_area::{SkiArea, UniqueId};
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::option_time_ser;
use segments::get_segments;
use use_lift::find_lift_usage;

mod segments;
mod use_lift;

#[cfg(test)]
mod segments_test;
#[cfg(test)]
mod test_util;
#[cfg(test)]
mod use_lift_test;

pub use segments::{Segment, Segments};
pub use use_lift::{LiftEnd, UseLift};

#[derive(Debug, Serialize)]
pub enum ActivityType<'s> {
    Unknown(()),
    UseLift(UseLift<'s>),
}

impl<'s> Default for ActivityType<'s> {
    fn default() -> Self {
        ActivityType::Unknown(())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Activity<'s> {
    #[serde(rename = "type")]
    pub type_: ActivityType<'s>,
    pub route: Segments,
    #[serde(with = "option_time_ser")]
    pub begin_time: Option<OffsetDateTime>,
    #[serde(with = "option_time_ser")]
    pub end_time: Option<OffsetDateTime>,
    pub length: f64,
}

impl<'s> Activity<'s> {
    fn new(type_: ActivityType<'s>, route: Segments) -> Self {
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

fn serialize_unique_id<S, T>(
    t: &&T,
    s: S,
) -> std::result::Result<S::Ok, S::Error>
where
    T: UniqueId,
    S: Serializer,
{
    s.serialize_str(t.get_unique_id())
}

pub type AnalyzedRoute<'s> = BoundedGeometry<Vec<Activity<'s>>>;

pub fn analyze_route<'s>(
    ski_area: &'s SkiArea,
    gpx: Gpx,
) -> Result<AnalyzedRoute<'s>> {
    let mut segments = get_segments(gpx)?;
    // println!("{:#?}", segments);
    let result = find_lift_usage(ski_area, take(&mut segments.item));
    Ok(BoundedGeometry {
        item: result,
        bounding_rect: segments.bounding_rect,
    })
}
