use geo::Point;
use serde::{Deserialize, Serialize};

use lift::parse_lift;
use piste::parse_pistes;

use crate::config::get_config;
use crate::error::{Error, ErrorType, Result};
use crate::osm_reader::{get_tag, Document};

mod bounded_geometry;
mod lift;
mod piste;

#[cfg(test)]
mod lift_test;
#[cfg(test)]
mod piste_test;

pub use bounded_geometry::BoundedGeometry;
pub use lift::Lift;
pub use piste::{Difficulty, Piste, PisteData, PisteMetadata};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PointWithElevation {
    pub point: Point,
    pub elevation: u32,
}

impl PointWithElevation {
    pub fn new(point: Point, elevation: u32) -> Self {
        Self { point, elevation }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkiArea {
    pub name: String,
    pub lifts: Vec<Lift>,
    pub pistes: Vec<Piste>,
}

impl SkiArea {
    pub fn parse(doc: &Document) -> Result<Self> {
        let mut names: Vec<String> = Vec::new();
        let mut lifts = Vec::new();
        let config = get_config();

        for (_id, way) in &doc.elements.ways {
            if get_tag(&way.tags, "landuse") == "winter_sports" {
                names.push(get_tag(&way.tags, "name").to_string());
            }
        }

        if names.len() == 0 {
            return Err(Error::new_s(
                ErrorType::InputError,
                "ski area entity not found",
            ));
        } else if names.len() > 1 {
            return Err(Error::new(
                ErrorType::InputError,
                format!("ambiguous ski area: {:?}", names),
            ));
        }

        for (id, way) in &doc.elements.ways {
            match parse_lift(&doc, &id, &way) {
                Err(e) => eprintln!("Error parsing way {}: {}", id, e),
                Ok(None) => (),
                Ok(Some(lift)) => lifts.push(lift),
            }
        }

        if config.is_v() {
            eprintln!("Found {} lifts.", lifts.len());
        }

        let pistes = parse_pistes(&doc);

        if config.is_v() {
            eprintln!("Found {} pistes.", pistes.len());
        }

        Ok(SkiArea {
            name: names.remove(0),
            lifts,
            pistes,
        })
    }
}
