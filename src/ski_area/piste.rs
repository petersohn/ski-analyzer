use geo::{BoundingRect, CoordNum, LineString, Polygon};

use std::str::FromStr;

use super::{BoundedGeometry, Difficulty, Piste, PisteMetadata};

use crate::config::get_config;
use crate::error::{InvalidInput, Result};
use crate::osm_reader::{get_tag, parse_way, Document, Tags, Way};

fn parse_metadata(tags: &Tags) -> Result<PisteMetadata> {
    let mut name = get_tag(&tags, "name");
    if name == "" {
        name = get_tag(&tags, "piste:name");
    }

    let mut ref_ = get_tag(&tags, "ref");
    if ref_ == "" {
        ref_ = get_tag(&tags, "piste:ref");
    }

    let difficulty_str = get_tag(&tags, "piste:difficulty");
    let difficulty = Difficulty::from_str(&difficulty_str).or(Err(
        InvalidInput::new(format!("invalid difficulty: {}", difficulty_str)),
    ))?;

    Ok(PisteMetadata {
        ref_: ref_.to_string(),
        name: name.to_string(),
        difficulty,
    })
}

struct PartialPiste<T, C = f64>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    metadata: PisteMetadata,
    geometry: BoundedGeometry<T, C>,
}

enum PartialPisteType {
    Line(PartialPiste<LineString>),
    Area(PartialPiste<Polygon>),
}

impl PartialPisteType {
    fn parse(doc: &Document, way: &Way) -> Result<Self> {
        let metadata = parse_metadata(&way.tags)?;
        let coords = parse_way(&doc, &way)?;
        let line = LineString::new(coords);

        Ok(if get_tag(&way.tags, "area") == "yes" {
            PartialPisteType::Area(PartialPiste {
                metadata,
                geometry: BoundedGeometry::new(Polygon::new(line, Vec::new()))?,
            })
        } else {
            PartialPisteType::Line(PartialPiste {
                metadata,
                geometry: BoundedGeometry::new(line)?,
            })
        })
    }
}

pub fn parse_pistes(doc: &Document) -> Vec<Piste> {
    let mut line_entities: Vec<PartialPiste<LineString>> = Vec::new();
    let mut area_entities: Vec<PartialPiste<Polygon>> = Vec::new();

    for (id, way) in &doc.elements.ways {
        if get_tag(&way.tags, "piste:type") != "downhill" {
            continue;
        }

        match PartialPisteType::parse(&doc, &way) {
            Err(err) => {
                eprintln!("{}: error parsing piste: {}", id, err);
                continue;
            }
            Ok(PartialPisteType::Line(line)) => {
                line_entities.push(line);
            }
            Ok(PartialPisteType::Area(area)) => {
                area_entities.push(area);
            }
        };
    }

    let config = get_config();
    if config.verbose {
        eprintln!(
            "Found {} linear and {} area piste entities.",
            line_entities.len(),
            area_entities.len()
        );
    }

    // let lines
    let pistes = Vec::new();
    pistes
}
