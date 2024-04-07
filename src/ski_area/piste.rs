use geo::{
    BoundingRect, Coord, CoordNum, HaversineLength, Intersects, LineString,
    MultiLineString, Polygon, Rect,
};

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use super::{BoundedGeometry, Difficulty, Piste, PisteMetadata};

use crate::config::get_config;
// use crate::error::{Error, ErrorType, Result};
use crate::collection::max_if;
use crate::error::Result;
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

#[derive(Default, Debug)]
struct PartialPistes {
    line_entities: Vec<BoundedGeometry<LineString>>,
    area_entities: Vec<BoundedGeometry<Polygon>>,
}

struct UnnamedPiste<T, C = f64>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    difficulty: Difficulty,
    geometry: BoundedGeometry<T, C>,
}

fn parse_partial_piste(
    doc: &Document,
    way: &Way,
    result: &mut HashMap<PisteMetadata, PartialPistes>,
    unnamed_lines: &mut Vec<UnnamedPiste<LineString>>,
    unnamed_areas: &mut Vec<UnnamedPiste<Polygon>>,
) -> Result<()> {
    let metadata = parse_metadata(&way.tags);
    let coords = parse_way(&doc, &way)?;
    let line = LineString::new(coords);
    let is_area = get_tag(&way.tags, "area") == "yes";

    if metadata.ref_ == "" && metadata.name == "" {
        if is_area {
            unnamed_areas.push(UnnamedPiste {
                difficulty: metadata.difficulty,
                geometry: BoundedGeometry::new(Polygon::new(line, Vec::new()))?,
            });
        } else {
            unnamed_lines.push(UnnamedPiste {
                difficulty: metadata.difficulty,
                geometry: BoundedGeometry::new(line)?,
            });
        }
        return Ok(());
    }

    let partial_piste = result.entry(metadata).or_default();

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
    HashMap<PisteMetadata, PartialPistes>,
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

fn find_differing_metadata<It>(metadatas: It)
where
    It: Iterator,
    It::Item: Borrow<PisteMetadata>,
{
    let mut map: HashMap<String, HashMap<String, HashSet<Difficulty>>> =
        HashMap::new();

    for metadata in metadatas {
        let m = metadata.borrow();
        let names = map.entry(m.ref_.clone()).or_default();
        let difficulties = names.entry(m.name.clone()).or_default();
        difficulties.insert(m.difficulty);
    }

    for (ref_, names) in map {
        if ref_ != "" && names.len() > 1 {
            eprintln!(
                "Multiple names for piste {}: {}",
                ref_,
                names
                    .keys()
                    .map(|k| k.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            );
        }

        for (name, difficulties) in names {
            if difficulties.len() > 1 {
                eprintln!(
                    "Multiple difficulties for piste {} {}: {}",
                    ref_,
                    name,
                    difficulties
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
            }
        }
    }
}

fn find_overlapping_pistes(pistes: &HashMap<PisteMetadata, PartialPistes>) {
    for (line_metadata, partial_pistes) in pistes {
        for line in &partial_pistes.line_entities {
            let length = line.item.haversine_length();

            let threshold = length / 2.0;

            for (area_metadata, partial_pistes2) in pistes {
                if area_metadata == line_metadata {
                    continue;
                }
                for area in &partial_pistes2.area_entities {
                    let intersection = get_intersection_length(area, line);
                    if intersection > threshold {
                        eprintln!(
                            "Line {:?} intersects area {:?} {}/{} m",
                            line_metadata, area_metadata, intersection, length
                        );
                    }
                }
            }
        }
    }
}

fn find_anomalies(pistes: &HashMap<PisteMetadata, PartialPistes>) {
    find_differing_metadata(pistes.keys());
    find_overlapping_pistes(&pistes);
}

fn create_pistes(
    partial_pistes: HashMap<PisteMetadata, PartialPistes>,
) -> Vec<Piste> {
    let mut result = Vec::new();
    result.reserve(partial_pistes.len());
    let config = get_config();
    for (metadata, piste) in partial_pistes.into_iter() {
        if piste.line_entities.len() == 0 && piste.area_entities.len() == 0 {
            if config.is_vv() {
                eprintln!(
                    "{} {}: no lines of areas.",
                    metadata.ref_, metadata.name
                );
            }
        }
        result.push(Piste {
            metadata,
            bounding_rect: piste
                .line_entities
                .iter()
                .map(|l| l.bounding_rect)
                .chain(piste.area_entities.iter().map(|a| a.bounding_rect))
                .reduce(|r1, r2| {
                    Rect::new(
                        Coord {
                            x: r1.min().x.min(r2.min().x),
                            y: r1.min().y.min(r2.min().y),
                        },
                        Coord {
                            x: r1.max().x.max(r2.max().x),
                            y: r1.max().y.max(r2.max().y),
                        },
                    )
                })
                .unwrap(),
        });
    }

    result
}

fn merge_empty_refs(
    input: HashMap<PisteMetadata, PartialPistes>,
) -> HashMap<PisteMetadata, PartialPistes> {
    let mut result: HashMap<PisteMetadata, PartialPistes> = HashMap::new();
    let mut refless: HashMap<PisteMetadata, PartialPistes> = HashMap::new();

    for (metadata, pistes) in input {
        if metadata.ref_ == "" {
            refless.insert(metadata, pistes);
        } else {
            result.insert(metadata, pistes);
        }
    }

    if refless.len() == 0 {
        return result;
    }

    for (metadata, pistes) in result.iter_mut() {
        if let Some(refless_pistes) = refless.get_mut(&PisteMetadata {
            ref_: String::new(),
            name: metadata.name.clone(),
            difficulty: metadata.difficulty,
        }) {
            pistes
                .line_entities
                .append(&mut refless_pistes.line_entities);
            pistes
                .area_entities
                .append(&mut refless_pistes.area_entities);
        }
    }

    for (metadata, pistes) in refless {
        if pistes.line_entities.len() != 0 || pistes.area_entities.len() != 0 {
            result.insert(metadata, pistes);
        }
    }

    result
}

pub fn parse_pistes(doc: &Document) -> Vec<Piste> {
    let (mut partial_pistes, mut unnamed_lines, mut unnamed_areas) =
        parse_partial_pistes(&doc);

    let config = get_config();

    if config.is_v() {
        eprintln!(
            "Found {} different pistes, {} linear and {} area unnamed piste entities.",
            partial_pistes.len(),
            unnamed_lines.len(),
            unnamed_areas.len()
        );
    }

    partial_pistes = merge_empty_refs(partial_pistes);

    if config.is_vv() {
        find_anomalies(&partial_pistes);
    }

    let mut changed = true;
    while changed {
        changed = false;
        let mut unnamed_areas2 = Vec::new();

        while let Some(area) = unnamed_areas.pop() {
            let target = max_if(
                partial_pistes.iter_mut(),
                |piste| {
                    piste.1.line_entities.iter().fold(0.0, |acc, line| {
                        acc + get_intersection_length(&area.geometry, &line)
                    })
                },
                |piste, len| {
                    *len > 0.0 && piste.0.difficulty == area.difficulty
                },
            );
            match target {
                Some((_, piste)) => {
                    piste.area_entities.push(area.geometry);
                    changed = true;
                }
                None => unnamed_areas2.push(area),
            }
        }

        let mut unnamed_lines2 = Vec::new();

        while let Some(line) = unnamed_lines.pop() {
            let target = max_if(
                partial_pistes.iter_mut(),
                |piste| {
                    piste.1.area_entities.iter().fold(0.0, |acc, area| {
                        acc + get_intersection_length(&area, &line.geometry)
                    })
                },
                |piste, len| {
                    *len > 0.0 && piste.0.difficulty == line.difficulty
                },
            );
            match target {
                Some((_, piste)) => {
                    piste.line_entities.push(line.geometry);
                    changed = true;
                }
                None => unnamed_lines2.push(line),
            }
        }

        unnamed_areas = unnamed_areas2.into();
        unnamed_lines = unnamed_lines2.into();
    }

    if config.is_v() {
        eprintln!(
            "Could not find named piste for {} linear and {} area entities.",
            unnamed_lines.len(),
            unnamed_areas.len()
        );
    }

    // let lines
    let pistes = Vec::new();
    pistes
}
