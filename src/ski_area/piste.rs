use geo::{BoundingRect, CoordNum, LineString, Polygon};

use std::collections::HashMap;
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

#[derive(PartialEq, Eq, Hash)]
struct PartialPisteId {
    id: String,
    is_ref: bool,
}

impl PartialPisteId {
    fn new(metadata: &PisteMetadata) -> Self {
        if metadata.ref_ != "" {
            PartialPisteId {
                id: metadata.ref_.clone(),
                is_ref: true,
            }
        } else {
            PartialPisteId {
                id: metadata.name.clone(),
                is_ref: false,
            }
        }
    }
}

struct PartialPistes {
    line_entities: Vec<PartialPiste<LineString>>,
    area_entities: Vec<PartialPiste<Polygon>>,
}

impl PartialPistes {
    fn new() -> Self {
        PartialPistes {
            line_entities: Vec::new(),
            area_entities: Vec::new(),
        }
    }
}

fn parse_partial_piste(doc: &Document, way: &Way, result: &mut HashMap<PartialPisteId, PartialPistes) {
}

fn parse_partial_pistes(
    doc: &Document,
) -> HashMap<PartialPisteId, PartialPistes> {
    let mut result = HashMap::new();

    for (id, way) in &doc.elements.ways {
        if get_tag(&way.tags, "piste:type") != "downhill" {
            continue;
        }

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

    result
}

pub fn parse_pistes(doc: &Document) -> Vec<Piste> {
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
