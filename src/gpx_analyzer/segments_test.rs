use crate::config::{set_config, Config};

use geo::point;
use gpx::Waypoint;

use rstest::{fixture, rstest};

fn way(input: &[(f64, f64, Option<f64>)]) -> Vec<Waypoint> {
    return input
        .iter()
        .map(|(x, y, h)| {
            let mut result = Waypoint::new(point! { x: *x, y: *y });
            result.hdop = h.clone();
            result
        })
        .collect();
}

struct Init;

#[fixture]
fn init() -> Init {
    match set_config(Config { verbose: 2 }) {
        Ok(()) => (),
        Err(_) => (),
    }
    Init {}
}
