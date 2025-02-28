use std::collections::HashMap;

use geo::{Intersects, LineString, Point, Polygon, Rect};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use lift::parse_lift;
use piste::parse_pistes;

use crate::config::get_config;
use crate::error::{convert_err, Error, ErrorType, Result};
use crate::osm_reader::{get_tag, parse_way, Document};
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::cancel::CancellationToken;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkiArea {
    pub metadata: SkiAreaMetadata,
    pub lifts: HashMap<String, Lift>,
    pub pistes: HashMap<String, Piste>,
    pub bounding_rect: Rect,
    #[serde(with = "time_ser")]
    pub date: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkiAreaMetadata {
    pub id: u64,
    pub name: String,
    pub outline: BoundedGeometry<Polygon>,
}

impl SkiAreaMetadata {
    pub fn find(doc: &Document) -> Result<Vec<SkiAreaMetadata>> {
        let mut result: Vec<SkiAreaMetadata> = doc
            .elements
            .ways
            .iter()
            .filter(|(_id, way)| {
                get_tag(&way.tags, "landuse") == "winter_sports"
            })
            .map(|(id, way)| -> Result<Self> {
                let outline = if !way.geometry.is_empty() {
                    way.geom_to_line_string()
                } else {
                    LineString::new(convert_err(
                        parse_way(doc, &way.nodes),
                        ErrorType::OSMError,
                    )?)
                };
                Ok(Self {
                    id: *id,
                    name: get_tag(&way.tags, "name").to_string(),
                    outline: BoundedGeometry::new(Polygon::new(
                        outline,
                        vec![],
                    ))?,
                })
            })
            .filter_map(|res| match res {
                Ok(value) => Some(value),
                Err(err) => {
                    eprintln!("Failed to calculate ski area metadata: {}", err);
                    None
                }
            })
            .collect();

        result.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
        Ok(result)
    }
}

fn find_lifts(
    cancel: &CancellationToken,
    doc: &Document,
) -> Result<HashMap<String, Lift>> {
    let mut result = HashMap::new();

    for (id, way) in &doc.elements.ways {
        cancel.check()?;
        match parse_lift(&doc, id, way) {
            Ok(Some(lift)) => {
                result.insert(id.to_string(), lift);
            }
            Ok(None) => (),
            Err(err) => {
                if err.get_type() == ErrorType::Cancelled {
                    return Err(err);
                }
                eprintln!("Error parsing way {}: {}", id, err);
            }
        };
    }

    Ok(result)
}

impl SkiArea {
    pub fn parse(cancel: &CancellationToken, doc: &Document) -> Result<Self> {
        let metadatas = SkiAreaMetadata::find(doc)?;
        let metadata = metadatas.into_iter().next().ok_or_else(|| {
            Error::new_s(ErrorType::InputError, "ski area entity not found")
        })?;

        let config = get_config();
        let lifts = find_lifts(cancel, doc)?;

        if config.is_v() {
            eprintln!("Found {} lifts.", lifts.len());
        }

        let pistes = parse_pistes(cancel, doc)?;

        if config.is_v() {
            eprintln!("Found {} pistes.", pistes.len());
        }

        SkiArea::new(metadata, lifts, pistes, doc.osm3s.timestamp_osm_base)
    }

    pub fn new(
        metadata: SkiAreaMetadata,
        lifts: HashMap<String, Lift>,
        pistes: HashMap<String, Piste>,
        date: OffsetDateTime,
    ) -> Result<Self> {
        let bounding_rect = union_rects_all(
            lifts
                .values()
                .map(|l| l.line.bounding_rect)
                .chain(pistes.values().map(|p| p.data.bounding_rect)),
        )
        .ok_or_else(|| Error::new_s(ErrorType::OSMError, "Empty ski area"))?;

        Ok(SkiArea {
            metadata,
            lifts,
            pistes,
            bounding_rect,
            date,
        })
    }

    pub fn clip_piste_lines(&mut self) {
        self.pistes.values_mut().for_each(|p| p.clip_lines());
    }

    pub fn get_closest_lift<'a>(
        &'a self,
        p: Point,
        limit: f64,
    ) -> Option<(&'a str, f64)> {
        let (lift_id, c) = self
            .lifts
            .iter()
            .filter(|(_, l)| l.line.expanded_rect(limit).intersects(&p))
            .filter_map(|(id, l)| Some((id, l.get_closest_point(p)?)))
            .min_by(|(_, c1), (_, c2)| c1.distance.total_cmp(&c2.distance))?;
        Some((lift_id, c.distance))
    }
}
