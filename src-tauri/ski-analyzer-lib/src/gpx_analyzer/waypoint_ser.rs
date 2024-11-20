use geo::Point;
use gpx::Waypoint;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::to_odt;
use crate::utils::option_time_ser;

#[derive(Serialize, Deserialize)]
pub struct WaypointDef {
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

impl Into<Waypoint> for WaypointDef {
    fn into(self) -> Waypoint {
        let mut result = Waypoint::new(self.point);
        result.speed = self.speed;
        result.time = self.time.map(|t| t.into());
        result.hdop = self.hdop;
        result
    }
}
