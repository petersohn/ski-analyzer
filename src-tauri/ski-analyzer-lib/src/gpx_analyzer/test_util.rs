use super::AnalyzedRoute;
use geo::point;
use gpx::Waypoint;

use std::fs::OpenOptions;

pub fn save_analyzed_route(piste: &AnalyzedRoute, filename: &str) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)
        .unwrap();
    serde_json::to_writer_pretty(file, &piste).unwrap();
}

pub fn wp(x: f64, y: f64, h: Option<f64>) -> Waypoint {
    let mut result = Waypoint::new(point! { x: x, y: y });
    result.hdop = h;
    result
}
