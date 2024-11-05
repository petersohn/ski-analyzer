use geo::{Haversine, Length, Line, Point};
use gpx::{Gpx, Time};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::error::Result;
use crate::ski_area::{SkiArea, UniqueId};
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::result::extract_option_result;
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

#[derive(Debug, Default)]
pub struct Activity<'s, 'g> {
    pub type_: ActivityType<'s>,
    pub route: Segments<'g>,
    pub begin_time: Option<OffsetDateTime>,
    pub end_time: Option<OffsetDateTime>,
    pub length: f64,
}

impl<'s, 'g> Activity<'s, 'g> {
    fn new(type_: ActivityType<'s>, route: Segments<'g>) -> Self {
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

impl<'s, 'g> Serialize for Activity<'s, 'g> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct WaypointDef {
            point: Point,
            time: Option<String>,
            hdop: Option<f64>,
            speed: Option<f64>,
        }

        let route: Vec<Vec<WaypointDef>> = self
            .route
            .iter()
            .map(|s| {
                s.iter()
                    .map(|wp| WaypointDef {
                        point: wp.point(),
                        time: wp.time.map(|t| t.format().unwrap()),
                        hdop: wp.hdop,
                        speed: wp.speed,
                    })
                    .collect()
            })
            .collect();

        let mut state = serializer.serialize_struct("Activity", 5)?;
        state.serialize_field("type", &self.type_)?;
        state.serialize_field("route", &route)?;
        state.serialize_field(
            "begin_time",
            &extract_option_result(
                self.begin_time.map(|t| t.format(&Iso8601::DEFAULT)),
            )
            .map_err(|e| serde::ser::Error::custom(e))?,
        )?;
        state.serialize_field(
            "end_time",
            &extract_option_result(
                self.end_time.map(|t| t.format(&Iso8601::DEFAULT)),
            )
            .map_err(|e| serde::ser::Error::custom(e))?,
        )?;
        state.serialize_field("length", &self.length)?;
        state.end()
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

pub type AnalyzedRoute<'s, 'g> = BoundedGeometry<Vec<Activity<'s, 'g>>>;

pub fn analyze_route<'s, 'g>(
    ski_area: &'s SkiArea,
    gpx: &'g Gpx,
) -> Result<AnalyzedRoute<'s, 'g>> {
    let segments = get_segments(&gpx)?;
    // println!("{:#?}", segments);
    let result = find_lift_usage(ski_area, &segments.item);
    Ok(BoundedGeometry {
        item: result,
        bounding_rect: segments.bounding_rect,
    })
}
