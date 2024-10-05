use crate::config::{set_config, Config};
use crate::osm_reader as r;

use geo::{coord, LineString};
use rstest::fixture;
use std::collections::HashMap;

pub fn node(x: f64, y: f64) -> r::Node {
    r::Node {
        lat: y,
        lon: x,
        tags: HashMap::new(),
    }
}

pub fn way(ids: &[u64]) -> r::Way {
    r::Way {
        nodes: Vec::from(ids),
        tags: HashMap::new(),
    }
}

fn create_tags(tags: &[(&str, &str)]) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for (key, value) in tags {
        result.insert(key.to_string(), value.to_string());
    }
    result
}

pub fn node_tags(x: f64, y: f64, tags: &[(&str, &str)]) -> r::Node {
    r::Node {
        lat: y,
        lon: x,
        tags: create_tags(tags),
    }
}

pub fn way_tags(ids: &[u64], tags: &[(&str, &str)]) -> r::Way {
    r::Way {
        nodes: Vec::from(ids),
        tags: create_tags(tags),
    }
}

pub fn line(points: &[(f64, f64)]) -> LineString {
    LineString(
        points
            .iter()
            .map(|(x, y)| coord! { x: *x, y: *y })
            .collect(),
    )
}

pub struct Init;

#[fixture]
pub fn init() -> Init {
    match set_config(Config { verbose: 2 }) {
        Ok(()) => (),
        Err(_) => (),
    }
    Init {}
}

#[macro_export]
macro_rules! assert_eq_pretty {
    ($left:expr, $right:expr) => {
        assert_eq!($left, $right, "\n{:#?}\n{:#?}", $left, $right);
    };
}

pub use assert_eq_pretty;
