use geo::{
    BooleanOps, BoundingRect, CoordNum, HaversineLength, Intersects,
    LineString, MultiLineString, Polygon,
};

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use super::{BoundedGeometry, Difficulty, Piste, PisteMetadata};

use crate::config::get_config;
use crate::error::{Error, ErrorType, Result};
use crate::iter::max_if;
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
    line_entities: Vec<BoundedGeometry<LineString>>,
    area_entities: Vec<BoundedGeometry<Polygon>>,
    ref_: String,
    names: HashSet<String>,
    difficulty: Difficulty,
}

impl PartialPistes {
    fn new(ref_: String) -> Self {
        PartialPistes {
            line_entities: Vec::new(),
            area_entities: Vec::new(),
            ref_,
            names: HashSet::new(),
            difficulty: Difficulty::Unknown,
        }
    }
}

struct UnnamedPiste<T, C = f64>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    id: u64,
    difficulty: Difficulty,
    geometry: BoundedGeometry<T, C>,
}

fn parse_partial_piste(
    doc: &Document,
    id: u64,
    way: &Way,
    result: &mut HashMap<PartialPisteId, PartialPistes>,
    unnamed_lines: &mut Vec<UnnamedPiste<LineString>>,
    unnamed_areas: &mut Vec<UnnamedPiste<Polygon>>,
) -> Result<()> {
    let metadata = parse_metadata(&way.tags);
    let coords = parse_way(&doc, &way)?;
    let line = LineString::new(coords);
    let is_area = get_tag(&way.tags, "area") == "yes";
    let key = PartialPisteId::new(&metadata);

    if key.id == "" {
        if is_area {
            unnamed_areas.push(UnnamedPiste {
                id,
                difficulty: metadata.difficulty,
                geometry: BoundedGeometry::new(Polygon::new(line, Vec::new()))?,
            });
        } else {
            unnamed_lines.push(UnnamedPiste {
                id,
                difficulty: metadata.difficulty,
                geometry: BoundedGeometry::new(line)?,
            });
        }
        return Ok(());
    }

    let partial_piste = match result.get_mut(&key) {
        Some(vec) => vec,
        None => {
            result
                .insert(key.clone(), PartialPistes::new(metadata.ref_.clone()));
            result.get_mut(&key).unwrap()
        }
    };

    if partial_piste.difficulty == Difficulty::Unknown {
        partial_piste.difficulty = metadata.difficulty;
    } else if partial_piste.difficulty != metadata.difficulty
        && get_config().is_vv()
    {
        eprintln!(
            "{} {} {}: Difficulty mismatch: previous={:?} new={:?}",
            id,
            metadata.ref_,
            metadata.name,
            partial_piste.difficulty,
            metadata.difficulty
        );
    }

    partial_piste.names.insert(metadata.name);

    if is_area {
        partial_piste
            .area_entities
            .push(BoundedGeometry::new(Polygon::new(line, Vec::new()))?);
    } else {
        partial_piste
            .line_entities
            .push(BoundedGeometry::new(line)?);
    }
    Ok(())
}

fn parse_partial_pistes(
    doc: &Document,
) -> (
    HashMap<PartialPisteId, PartialPistes>,
    Vec<UnnamedPiste<LineString>>,
    Vec<UnnamedPiste<Polygon>>,
) {
    let mut result = HashMap::new();
    let mut unnamed_lines = Vec::new();
    let mut unnamed_areas = Vec::new();

    for (id, way) in &doc.elements.ways {
        if get_tag(&way.tags, "piste:type") != "downhill" {
            continue;
        }

        if let Err(err) = parse_partial_piste(
            &doc,
            *id,
            &way,
            &mut result,
            &mut unnamed_lines,
            &mut unnamed_areas,
        ) {
            eprintln!("{}: error parsing piste: {}", id, err);
        }
    }

    (result, unnamed_lines, unnamed_areas)
}

fn get_intersection_length(
    area: &BoundedGeometry<Polygon>,
    line: &BoundedGeometry<LineString>,
) -> f64 {
    if !area.bounding_rect.intersects(&line.bounding_rect) {
        return 0.0;
    }

    let intersection = area
        .item
        .clip(&MultiLineString::new(vec![line.item.clone()]), false);
    intersection.haversine_length()
}

pub fn parse_pistes(doc: &Document) -> Vec<Piste> {
    let (mut partial_pistes, mut unnamed_lines, mut unnamed_areas) =
        parse_partial_pistes(&doc);

    let config = get_config();

    if config.is_v() {
        eprintln!(
            "Found {} linear and {} area unnamed piste entities.",
            unnamed_lines.len(),
            unnamed_areas.len()
        );
    }
    let mut unnamed_areas2 = Vec::new();

    while let Some(area) = unnamed_areas.pop() {
        let target = max_if(
            partial_pistes.values_mut(),
            |piste| {
                piste.line_entities.iter().fold(0.0, |acc, line| {
                    acc + get_intersection_length(&area.geometry, &line)
                })
            },
            |piste, len| *len > 0.0 && piste.difficulty == area.difficulty,
        );
        match target {
            Some(piste) => piste.area_entities.push(area.geometry),
            None => unnamed_areas2.push(area),
        }
    }

    let mut unnamed_lines2 = Vec::new();

    while let Some(line) = unnamed_lines.pop() {
        let target = max_if(
            partial_pistes.values_mut(),
            |piste| {
                piste.area_entities.iter().fold(0.0, |acc, area| {
                    acc + get_intersection_length(&area, &line.geometry)
                })
            },
            |piste, len| *len > 0.0 && piste.difficulty == line.difficulty,
        );
        match target {
            Some(piste) => piste.line_entities.push(line.geometry),
            None => unnamed_lines2.push(line),
        }
    }
    if config.is_v() {
        eprintln!(
            "Could not find named piste for {} linear and {} area entities.",
            unnamed_lines2.len(),
            unnamed_areas2.len()
        );
    }

    if config.is_v() {
        eprintln!("Found {} differently named pistes.", partial_pistes.len());
    }

    // let lines
    let pistes = Vec::new();
    pistes
}
