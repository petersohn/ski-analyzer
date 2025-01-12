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
    elevation: Option<f64>,
    hdop: Option<f64>,
    vdop: Option<f64>,
}

impl From<Waypoint> for WaypointDef {
    fn from(wp: Waypoint) -> Self {
        WaypointDef {
            point: wp.point(),
            speed: wp.speed,
            time: to_odt(wp.time),
            elevation: wp.elevation,
            hdop: wp.hdop,
            vdop: wp.vdop,
        }
    }
}

impl Into<Waypoint> for WaypointDef {
    fn into(self) -> Waypoint {
        let mut result = Waypoint::new(self.point);
        result.speed = self.speed;
        result.time = self.time.map(|t| t.into());
        result.elevation = self.elevation;
        result.hdop = self.hdop;
        result.vdop = self.vdop;
        result
    }
}
