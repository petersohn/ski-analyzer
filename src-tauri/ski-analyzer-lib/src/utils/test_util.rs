use crate::config::{set_config, Config};
use crate::osm_reader as r;
use crate::ski_area::{SkiArea, SkiAreaMetadata};
use crate::utils::bounded_geometry::BoundedGeometry;

use geo::{coord, LineString, Polygon, Rect};
use rstest::fixture;

use std::collections::HashMap;
use std::fs::OpenOptions;

pub fn node(x: f64, y: f64) -> r::Node {
    r::Node {
        coordinate: r::Coordinate { lat: y, lon: x },
        tags: HashMap::new(),
    }
}

pub fn way(ids: &[u64]) -> r::Way {
    r::Way {
        nodes: Vec::from(ids),
        tags: HashMap::new(),
        geometry: vec![],
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
        coordinate: r::Coordinate { lat: y, lon: x },
        tags: create_tags(tags),
    }
}

pub fn way_tags(ids: &[u64], tags: &[(&str, &str)]) -> r::Way {
    r::Way {
        nodes: Vec::from(ids),
        tags: create_tags(tags),
        geometry: vec![],
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

pub fn save_ski_area(piste: &SkiArea, filename: &str) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)
        .unwrap();
    serde_json::to_writer_pretty(file, &piste).unwrap();
}

pub fn create_ski_area_metadata(name: String) -> SkiAreaMetadata {
    SkiAreaMetadata {
        name,
        id: 0,
        outline: BoundedGeometry {
            item: Polygon::new(LineString::new(vec![]), vec![]),
            bounding_rect: Rect::new(
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 0.0, y: 0.0 },
            ),
        },
    }
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
