use geo::Point;
use gpx::Waypoint;
use serde::{Serialize, Serializer};
use time::OffsetDateTime;

use super::segments::Segments;
use super::to_odt;
use crate::utils::option_time_ser;

#[derive(Serialize)]
struct WaypointDef {
    pub point: Point,
    speed: Option<f64>,
    #[serde(with = "option_time_ser")]
    time: Option<OffsetDateTime>,
    hdop: Option<f64>,
}

impl From<Waypoint> for WaypointDef {
    fn from(wp: Waypoint) -> Self {
        WaypointDef {
            point: wp.point(),
            speed: wp.speed,
            time: to_odt(wp.time),
            hdop: wp.hdop,
        }
    }
}

pub fn serialize<S>(
    segments: &Segments,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let data: Vec<Vec<WaypointDef>> = segments
        .into_iter()
        .map(|s| s.into_iter().map(|wp| wp.clone().into()).collect())
        .collect();
    data.serialize(serializer)
}
