use geo::{Intersects, Point, Rect};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use lift::parse_lift;
use piste::parse_pistes;

use crate::config::get_config;
use crate::error::{Error, ErrorType, Result};
use crate::osm_reader::{get_tag, Document};
use crate::utils::rect::union_rects_all;
use crate::utils::time_ser;

mod lift;
mod piste;

#[cfg(test)]
mod lift_test;
#[cfg(test)]
mod piste_test;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkiArea {
    pub name: String,
    pub lifts: Vec<Lift>,
    pub pistes: Vec<Piste>,
    pub bounding_rect: Rect,
    #[serde(with = "time_ser")]
    pub date: OffsetDateTime,
}

fn find_name(doc: &Document) -> Result<String> {
    let mut names: Vec<String> = doc
        .elements
        .ways
        .iter()
        .filter(|(_id, way)| get_tag(&way.tags, "landuse") == "winter_sports")
        .map(|(_id, way)| get_tag(&way.tags, "name").to_string())
        .collect();

    if names.len() > 1 {
        Err(Error::new(
            ErrorType::InputError,
            format!("ambiguous ski area: {:?}", names),
        ))
    } else {
        names.pop().ok_or_else(|| {
            Error::new_s(ErrorType::InputError, "ski area entity not found")
        })
    }
}

fn find_lifts(doc: &Document) -> Vec<Lift> {
    doc.elements
        .ways
        .iter()
        .filter_map(|(id, way)| {
            parse_lift(&doc, &id, &way).unwrap_or_else(|e| {
                eprintln!("Error parsing way {}: {}", id, e);
                None
            })
        })
        .collect()
}

impl SkiArea {
    pub fn parse(doc: &Document) -> Result<Self> {
        let name = find_name(doc)?;
        let config = get_config();
        let lifts = find_lifts(doc);

        if config.is_v() {
            eprintln!("Found {} lifts.", lifts.len());
        }

        let pistes = parse_pistes(doc);

        if config.is_v() {
            eprintln!("Found {} pistes.", pistes.len());
        }

        SkiArea::new(name, lifts, pistes, doc.osm3s.timestamp_osm_base)
    }

    pub fn new(
        name: String,
        lifts: Vec<Lift>,
        pistes: Vec<Piste>,
        date: OffsetDateTime,
    ) -> Result<Self> {
        let bounding_rect = union_rects_all(
            lifts
                .iter()
                .map(|l| l.line.bounding_rect)
                .chain(pistes.iter().map(|p| p.data.bounding_rect)),
        )
        .ok_or_else(|| Error::new_s(ErrorType::OSMError, "Empty ski area"))?;

        Ok(SkiArea {
            name,
            lifts,
            pistes,
            bounding_rect,
            date,
        })
    }

    pub fn clip_piste_lines(&mut self) {
        self.pistes.iter_mut().for_each(|p| p.clip_lines());
    }

    pub fn get_closest_lift<'a>(
        &'a self,
        p: Point,
        limit: f64,
    ) -> Option<(&'a Lift, f64)> {
        let (lift, c) = self
            .lifts
            .iter()
            .filter(|l| l.line.expanded_rect(limit).intersects(&p))
            .filter_map(|l| Some((l, l.get_closest_point(p)?)))
            .min_by(|(_, c1), (_, c2)| c1.distance.total_cmp(&c2.distance))?;
        Some((lift, c.distance))
    }
}
