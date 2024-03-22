use geo::{
    BooleanOps, BoundingRect, CoordNum, HaversineLength, Intersects,
    LineString, MultiLineString, Polygon,
};

use std::collections::HashMap;
use std::str::FromStr;

use super::{BoundedGeometry, Difficulty, Piste, PisteMetadata};

use crate::config::get_config;
use crate::error::{Error, ErrorType, Result};
use crate::osm_reader::{get_tag, parse_way, Document, Tags, Way};

fn parse_metadata(tags: &Tags) -> PisteMetadata {
    let mut name = get_tag(&tags, "name");
    if name == "" {
        name = get_tag(&tags, "piste:name");
    }

    let mut ref_ = get_tag(&tags, "ref");
    if ref_ == "" {
        ref_ = get_tag(&tags, "piste:ref");
    }

    let difficulty_str = get_tag(&tags, "piste:difficulty");

    let difficulty = match Difficulty::from_str(&difficulty_str) {
        Ok(difficulty) => difficulty,
        Err(_) => {
            eprintln!(
                "{} {}: invalid difficulty: {}",
                name, ref_, difficulty_str
            );
            Difficulty::Unknown
        }
    };

    PisteMetadata {
        ref_: ref_.to_string(),
        name: name.to_string(),
        difficulty,
    }
}

struct PartialPiste<T, C = f64>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    metadata: PisteMetadata,
    geometry: BoundedGeometry<T, C>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
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

    fn empty() -> Self {
        PartialPisteId {
            id: String::new(),
            is_ref: false,
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

fn parse_partial_piste(
    doc: &Document,
    way: &Way,
    result: &mut HashMap<PartialPisteId, PartialPistes>,
) -> Result<()> {
    let metadata = parse_metadata(&way.tags);
    let coords = parse_way(&doc, &way)?;
    let line = LineString::new(coords);
    let key = PartialPisteId::new(&metadata);
    let partial_piste = match result.get_mut(&key) {
        Some(vec) => vec,
        None => {
            result.insert(key.clone(), PartialPistes::new());
            result.get_mut(&key).unwrap()
        }
    };

    if get_tag(&way.tags, "area") == "yes" {
        partial_piste.line_entities.push(PartialPiste {
            metadata,
            geometry: BoundedGeometry::new(line)?,
        });
    } else {
        partial_piste.area_entities.push(PartialPiste {
            metadata,
            geometry: BoundedGeometry::new(Polygon::new(line, Vec::new()))?,
        });
    }
    Ok(())
}

fn parse_partial_pistes(
    doc: &Document,
) -> HashMap<PartialPisteId, PartialPistes> {
    let mut result = HashMap::new();

    for (id, way) in &doc.elements.ways {
        if get_tag(&way.tags, "piste:type") != "downhill" {
            continue;
        }

        if let Err(err) = parse_partial_piste(&doc, &way, &mut result) {
            eprintln!("{}: error parsing piste: {}", id, err);
        }
    }

    result
}

fn get_intersection_length(
    area: &PartialPiste<Polygon>,
    line: &PartialPiste<LineString>,
) -> f64 {
    if !area
        .geometry
        .bounding_rect
        .intersects(&line.geometry.bounding_rect)
    {
        return 0.0;
    }

    let intersection = area.geometry.item.clip(
        &MultiLineString::new(vec![line.geometry.item.clone()]),
        false,
    );
    intersection.haversine_length()
}

pub fn parse_pistes(doc: &Document) -> Vec<Piste> {
    let mut partial_pistes = parse_partial_pistes(&doc);

    let config = get_config();

    if let Some(mut unnamed) = partial_pistes.remove(&PartialPisteId::empty()) {
        if config.verbose {
            eprintln!(
                "Found {} linear and {} area unnamed piste entities.",
                unnamed.line_entities.len(),
                unnamed.area_entities.len()
            );
        }
        let mut unnamed_areas: Vec<PartialPiste<Polygon>> = Vec::new();

        while let Some(area) = unnamed.area_entities.pop() {
            let mut target: Option<&mut PartialPistes> = None;
            let mut max_len: f64 = 0.0;
            for piste in partial_pistes.values_mut() {
                let len = piste.line_entities.iter().fold(0.0, |acc, line| {
                    acc + get_intersection_length(&area, &line)
                });
                if len > 0.0 && len > max_len {
                    target = Some(piste);
                    max_len = len;
                }
            }

            match target {
                Some(piste) => piste.area_entities.push(area),
                None => unnamed_areas.push(area),
            }
        }

        let mut unnamed_lines: Vec<PartialPiste<LineString>> = Vec::new();

        while let Some(line) = unnamed.line_entities.pop() {
            let mut target: Option<&mut PartialPistes> = None;
            let mut max_len: f64 = 0.0;
            for piste in partial_pistes.values_mut() {
                let len = piste.area_entities.iter().fold(0.0, |acc, area| {
                    acc + get_intersection_length(&area, &line)
                });
                if len > 0.0 && len > max_len {
                    target = Some(piste);
                    max_len = len;
                }
            }

            match target {
                Some(piste) => piste.line_entities.push(line),
                None => unnamed_lines.push(line),
            }
        }
    }

    if config.verbose {
        eprintln!("Found {} differently named pistes.", partial_pistes.len());
    }

    // let lines
    let pistes = Vec::new();
    pistes
}
