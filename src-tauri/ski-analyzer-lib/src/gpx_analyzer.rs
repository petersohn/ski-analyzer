use geo::Point;
use gpx::{Gpx, Time, Waypoint};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::ski_area::{SkiArea, UniqueId};
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

#[derive(Debug, Default, Serialize)]
pub enum ActivityType<'s> {
    #[default]
    Unknown,
    UseLift(UseLift<'s>),
}

#[derive(Debug, Default)]
pub struct Activity<'s, 'g> {
    pub type_: ActivityType<'s>,
    pub route: Segments<'g>,
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
        }

        let route: Vec<Vec<WaypointDef>> = self
            .route
            .iter()
            .map(|s| {
                s.iter()
                    .map(|wp| WaypointDef {
                        point: wp.point(),
                        time: wp.time.map(|t| t.format().unwrap()),
                    })
                    .collect()
            })
            .collect();

        let mut state = serializer.serialize_struct("Activity", 2)?;
        state.serialize_field("type", &self.type_)?;
        state.serialize_field("route", &route)?;
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

fn get_time(wp: &Waypoint) -> Option<String> {
    wp.time.map(|t| t.format().unwrap())
}

pub fn analyze_route<'s, 'g>(
    ski_area: &'s SkiArea,
    gpx: &'g Gpx,
) -> Vec<Activity<'s, 'g>> {
    let segments = get_segments(&gpx);
    // println!("{:#?}", segments);
    let result = find_lift_usage(ski_area, &segments);
    result
}
