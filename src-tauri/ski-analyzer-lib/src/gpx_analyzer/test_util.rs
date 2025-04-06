use super::Activity;
use crate::utils::{
    bounded_geometry::BoundedGeometry, json::save_to_file,
    rect::union_rects_all,
};
use geo::{point, Rect};
use gpx::Waypoint;

pub fn save_analyzed_route(result: &Vec<Activity>, filename: &str) {
    let bounding_rect = union_rects_all(
        result
            .iter()
            .map(|a| {
                a.route
                    .0
                    .iter()
                    .flatten()
                    .map(|wp| Rect::new(wp.point(), wp.point()))
            })
            .flatten(),
    )
    .unwrap();
    let analyzed_route = BoundedGeometry {
        item: result,
        bounding_rect,
    };
    save_to_file(&analyzed_route, filename).unwrap();
}

pub fn wp(x: f64, y: f64, h: Option<f64>) -> Waypoint {
    let mut result = Waypoint::new(point! { x: x, y: y });
    result.hdop = h;
    result
}
