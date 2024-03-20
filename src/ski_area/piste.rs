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
    let difficulty =
        Difficulty::from_str(&difficulty_str).or(Err(Error::new(
            ErrorType::OSMError,
            format!("invalid difficulty: {}", difficulty_str),
        )))?;

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
    let metadata = parse_metadata(&way.tags)?;
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
    lines: &Vec<PartialPiste<LineString>>,
) -> f64 {
    let mut result = 0.0;

    for line in lines {
        if !area
            .geometry
            .bounding_rect
            .intersects(&line.geometry.bounding_rect)
        {
            continue;
        }

        let intersection = area.geometry.item.clip(
            &MultiLineString::new(vec![line.geometry.item.clone()]),
            false,
        );
        result += intersection.haversine_length();
    }

    result
}

pub fn parse_pistes(doc: &Document) -> Vec<Piste> {
    let mut partial_pistes = parse_partial_pistes(&doc);
    let mut unnamed = partial_pistes.remove(&PartialPisteId::empty());

    let config = get_config();
    if config.verbose {
        let count = match &unnamed {
            None => partial_pistes.len(),
            Some(_) => partial_pistes.len() - 1,
        };
        eprintln!("Found {} differently named pistes.", count);
    }

    if let Some(u) = &mut unnamed {
        if config.verbose {
            eprintln!(
                "Found {} linear and {} area unnamed piste entities.",
                u.line_entities.len(),
                u.area_entities.len()
            );
        }
        let mut unnamed_areas: Vec<PartialPiste<Polygon>> = Vec::new();

        while let Some(area) = u.area_entities.pop() {
            let mut target: Option<&mut PartialPistes> = None;
            let mut max_len: f64 = 0.0;
            for piste in partial_pistes.values_mut() {
                let len = get_intersection_length(&area, &piste.line_entities);
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
    }

    // let lines
    let pistes = Vec::new();
    pistes
}
