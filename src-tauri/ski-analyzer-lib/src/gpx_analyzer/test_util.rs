use super::AnalyzedRoute;
use crate::utils::json::save_to_file;
use geo::point;
use gpx::Waypoint;

pub fn save_analyzed_route(piste: &AnalyzedRoute, filename: &str) {
    save_to_file(piste, filename).unwrap();
}

pub fn wp(x: f64, y: f64, h: Option<f64>) -> Waypoint {
    let mut result = Waypoint::new(point! { x: x, y: y });
    result.hdop = h;
    result
}
