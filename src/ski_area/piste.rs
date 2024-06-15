use geo::{
    BooleanOps, BoundingRect, Coord, CoordNum, HaversineLength, Intersects,
    LineString, MultiLineString, MultiPolygon, Polygon, Rect,
};

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use super::{BoundedGeometry, Difficulty, Piste, PisteData, PisteMetadata};

use crate::config::get_config;
// use crate::error::{Error, ErrorType, Result};
use crate::collection::max_if;
use crate::error::Result;
use crate::multipolygon::parse_multipolygon;
use crate::osm_reader::{get_tag, parse_way, Document, Tags, Way};

fn parse_metadata(tags: &Tags) -> PisteMetadata {
    let mut name = get_tag(&tags, "piste:name");
    if name == "" {
        name = get_tag(&tags, "name");
    }

    let mut ref_ = get_tag(&tags, "piste:ref");
    if ref_ == "" {
        ref_ = get_tag(&tags, "ref");
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
    area_entities: Vec<BoundedGeometry<MultiPolygon>>,
}

struct UnnamedPiste<T, C = f64>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    difficulty: Difficulty,
    geometry: BoundedGeometry<T, C>,
}

enum PisteGeometry {
    Line(BoundedGeometry<LineString>),
    Area(BoundedGeometry<MultiPolygon>),
}

fn add_piste(
    metadata: PisteMetadata,
    geometry: PisteGeometry,
    result: &mut HashMap<PisteMetadata, PartialPistes>,
    unnamed_lines: &mut Vec<UnnamedPiste<LineString>>,
    unnamed_areas: &mut Vec<UnnamedPiste<MultiPolygon>>,
) {
    if metadata.ref_ == "" && metadata.name == "" {
        match geometry {
            PisteGeometry::Area(a) => {
                unnamed_areas.push(UnnamedPiste {
                    difficulty: metadata.difficulty,
                    geometry: a,
                });
            }
            PisteGeometry::Line(l) => {
                unnamed_lines.push(UnnamedPiste {
                    difficulty: metadata.difficulty,
                    geometry: l,
                });
            }
        };
        return;
    }

    let partial_piste = result.entry(metadata).or_default();

    match geometry {
        PisteGeometry::Area(a) => partial_piste.area_entities.push(a),
        PisteGeometry::Line(l) => partial_piste.line_entities.push(l),
    }
}

fn parse_partial_piste(doc: &Document, way: &Way) -> Result<PisteGeometry> {
    let coords = parse_way(&doc, &way.nodes)?;
    let line = LineString::new(coords);

    let geometry = if get_tag(&way.tags, "area") == "yes" {
        PisteGeometry::Area(BoundedGeometry::new(MultiPolygon::new(vec![
            Polygon::new(line, Vec::new()),
        ]))?)
    } else {
        PisteGeometry::Line(BoundedGeometry::new(line)?)
    };

    Ok(geometry)
}

fn merge_route_metadata(
    id: u64,
    tags: &Tags,
    route_index: &HashMap<u64, PisteMetadata>,
) -> PisteMetadata {
    let mut metadata = parse_metadata(tags);

    if let Some(md) = route_index.get(&id) {
        let mut discrepancy = false;
        if !md.ref_.is_empty() {
            if metadata.ref_.is_empty() {
                metadata.ref_ = md.ref_.clone();
            } else if metadata.ref_ != md.ref_ {
                discrepancy = true;
            }
        }
        if !md.name.is_empty() {
            if metadata.name.is_empty() {
                metadata.name = md.name.clone();
            } else if metadata.name != md.name {
                discrepancy = true;
            }
        }
        if md.difficulty != Difficulty::Unknown {
            if metadata.difficulty == Difficulty::Unknown {
                metadata.difficulty = md.difficulty.clone();
            } else if metadata.difficulty != md.difficulty {
                discrepancy = true;
            }
        }

        if discrepancy && get_config().is_vv() {
            eprintln!(
                "Route metadata discrepancy: route={:?}, way={:?}",
                md, metadata
            );
        }
    }

    metadata
}

fn parse_partial_pistes(
    doc: &Document,
) -> (
    HashMap<PisteMetadata, PartialPistes>,
    Vec<UnnamedPiste<LineString>>,
    Vec<UnnamedPiste<MultiPolygon>>,
) {
    let mut result = HashMap::new();
    let mut unnamed_lines = Vec::new();
    let mut unnamed_areas = Vec::new();
    let mut route_index: HashMap<u64, PisteMetadata> = HashMap::new();

    let config = get_config();

    for (id, relation) in &doc.elements.relations {
        if get_tag(&relation.tags, "type") != "route"
            || get_tag(&relation.tags, "route") != "piste"
            || get_tag(&relation.tags, "piste:type") != "downhill"
        {
            continue;
        }

        let metadata = parse_metadata(&relation.tags);
        if metadata.ref_.is_empty()
            && metadata.name.is_empty()
            && metadata.difficulty == Difficulty::Unknown
        {
            if config.is_vv() {
                eprintln!("{}: route has no meaningful metadata.", id);
            }
            continue;
        }

        for member in &relation.members.ways {
            route_index.insert(member.ref_, metadata.clone());
        }
    }

    for (id, way) in &doc.elements.ways {
        if get_tag(&way.tags, "piste:type") != "downhill" {
            continue;
        }

        match parse_partial_piste(&doc, &way) {
            Ok(geometry) => add_piste(
                merge_route_metadata(*id, &way.tags, &route_index),
                geometry,
                &mut result,
                &mut unnamed_lines,
                &mut unnamed_areas,
            ),
            Err(err) => eprintln!("{}: error parsing piste: {}", id, err),
        };
    }

    for (id, relation) in &doc.elements.relations {
        if get_tag(&relation.tags, "type") != "multipolygon"
            || get_tag(&relation.tags, "piste:type") != "downhill"
        {
            continue;
        }

        match parse_multipolygon(&doc, &relation) {
            Ok(mp) => {
                let metadata = parse_metadata(&relation.tags);
                for p in mp.0 {
                    match BoundedGeometry::new(MultiPolygon::new(vec![p])) {
                        Ok(geometry) => add_piste(
                            metadata.clone(),
                            PisteGeometry::Area(geometry),
                            &mut result,
                            &mut unnamed_lines,
                            &mut unnamed_areas,
                        ),
                        Err(err) => {
                            eprintln!("{}: {}", id, err)
                        }
                    };
                }
            }
            Err(err) => {
                eprintln!("{}: error parsing multipolygon: {}", id, err)
            }
        };
    }

    (result, unnamed_lines, unnamed_areas)
}

fn get_intersection_length(
    area: &BoundedGeometry<MultiPolygon>,
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

fn find_overlapping_pistes(pistes: &Vec<Piste>) {
    for (i, piste) in pistes.iter().enumerate() {
        let length = piste.data.lines.haversine_length();
        let threshold = length / 2.0;

        for (j, piste2) in pistes.iter().enumerate() {
            if i == j
                || !piste
                    .data
                    .bounding_rect
                    .intersects(&piste2.data.bounding_rect)
            {
                continue;
            }
            let intersection = piste2
                .data
                .areas
                .clip(&piste.data.lines, false)
                .haversine_length();
            if intersection > threshold {
                eprintln!(
                    "Line {:?} intersects area {:?} {:.0}/{:.0} m",
                    piste.metadata, piste2.metadata, intersection, length
                );
            }
        }
    }
}

fn find_anomalies(pistes: &Vec<Piste>) {
    find_differing_metadata(pistes.iter().map(|p| &p.metadata));
    find_overlapping_pistes(&pistes);
}

fn union_rects(r1: Rect, r2: Rect) -> Rect {
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
}

fn line_to_piste(line: BoundedGeometry<LineString>) -> PisteData {
    PisteData {
        bounding_rect: line.bounding_rect,
        areas: MultiPolygon::new(Vec::new()),
        lines: MultiLineString::new(vec![line.item]),
    }
}

fn area_to_piste(area: BoundedGeometry<MultiPolygon>) -> PisteData {
    PisteData {
        bounding_rect: area.bounding_rect,
        areas: area.item,
        lines: MultiLineString::new(Vec::new()),
    }
}

fn merge_pistes(target: &mut PisteData, source: &mut PisteData) {
    target.lines.0.append(&mut source.lines.0);
    target.areas.0.append(&mut source.areas.0);
    target.bounding_rect =
        union_rects(target.bounding_rect, source.bounding_rect);
}

fn merge_intersecting_pistes(pistes: &mut Vec<PisteData>) {
    let mut i = 0;
    while i < pistes.len() - 1 {
        let mut changed = false;
        let mut j = i + 1;
        while j < pistes.len() {
            if pistes[i].intersects(&pistes[j]) {
                let mut item = pistes.remove(j);
                merge_pistes(&mut pistes[i], &mut item);
                changed = true;
            } else {
                j += 1;
            }
        }

        if !changed {
            i += 1;
        }
    }
}

fn create_pistes(
    partial_pistes: HashMap<PisteMetadata, PartialPistes>,
) -> Vec<Piste> {
    let mut result = Vec::new();
    result.reserve(partial_pistes.len());
    let config = get_config();
    for (metadata, partial_piste) in partial_pistes {
        if partial_piste.line_entities.len() == 0
            && partial_piste.area_entities.len() == 0
        {
            if config.is_vv() {
                eprintln!(
                    "{} {}: no lines of areas.",
                    metadata.ref_, metadata.name
                );
            }
            continue;
        }

        let mut datas: Vec<PisteData> = partial_piste
            .line_entities
            .into_iter()
            .map(|line| line_to_piste(line))
            .chain(
                partial_piste
                    .area_entities
                    .into_iter()
                    .map(|area| area_to_piste(area)),
            )
            .collect();
        merge_intersecting_pistes(&mut datas);

        if datas.len() > 1 {
            if config.is_vv() {
                eprintln!(
                    "{} {}: piste has {} disjunct parts",
                    metadata.ref_,
                    metadata.name,
                    datas.len()
                );
            }

            result.append(
                &mut datas
                    .into_iter()
                    .map(|data| Piste {
                        metadata: metadata.clone(),
                        data,
                    })
                    .collect(),
            );
        } else {
            result.push(Piste {
                metadata,
                data: datas.into_iter().next().unwrap(),
            });
        }
    }

    result
}

fn merge_unnamed_pistes(
    unnamed_lines: Vec<UnnamedPiste<LineString>>,
    unnamed_areas: Vec<UnnamedPiste<MultiPolygon>>,
) -> Vec<Piste> {
    let mut pistes: HashMap<Difficulty, Vec<PisteData>> = HashMap::new();
    for line in unnamed_lines {
        let v = pistes.entry(line.difficulty).or_default();
        v.push(line_to_piste(line.geometry));
    }
    for area in unnamed_areas {
        let v = pistes.entry(area.difficulty).or_default();
        v.push(area_to_piste(area.geometry));
    }

    for mut datas in pistes.values_mut() {
        merge_intersecting_pistes(&mut datas);
    }

    let it = pistes.into_iter().map(|(difficulty, datas)| {
        datas.into_iter().map(move |data| Piste {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: String::new(),
                difficulty,
            },
            data,
        })
    });
    it.flatten().collect()
}

fn handle_unnamed_entities(
    mut unnamed_lines: Vec<UnnamedPiste<LineString>>,
    mut unnamed_areas: Vec<UnnamedPiste<MultiPolygon>>,
    partial_pistes: &mut HashMap<PisteMetadata, PartialPistes>,
) -> Vec<Piste> {
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

    let config = get_config();
    if config.is_v() {
        eprintln!(
            "Could not find named piste for {} linear and {} area entities.",
            unnamed_lines.len(),
            unnamed_areas.len()
        );
    }

    let result = merge_unnamed_pistes(unnamed_lines, unnamed_areas);

    if config.is_v() {
        eprintln!("Calculated {} distinct unnamed pistes", result.len());
    }

    result
}

fn merge_empty_refs(input: Vec<Piste>) -> Vec<Piste> {
    let mut result: Vec<Piste> = Vec::new();
    let mut refless: HashMap<PisteMetadata, Vec<PisteData>> = HashMap::new();

    for piste in input {
        if piste.metadata.ref_ == "" {
            let datas = refless.entry(piste.metadata).or_default();
            datas.push(piste.data);
        } else {
            result.push(piste);
        }
    }

    if refless.len() == 0 {
        return result;
    }

    for piste in result.iter_mut() {
        if let Some(refless_pistes) = refless.get_mut(&PisteMetadata {
            ref_: String::new(),
            name: piste.metadata.name.clone(),
            difficulty: piste.metadata.difficulty,
        }) {
            for refless_piste in refless_pistes.iter_mut() {
                if piste.data.intersects(refless_piste) {
                    merge_pistes(&mut piste.data, refless_piste);
                }
            }
        }
    }

    for (metadata, datas) in refless {
        if datas.len() == 1 {
            if datas[0].lines.0.len() != 0 || datas[0].areas.0.len() != 0 {
                result.push(Piste {
                    metadata,
                    data: datas.into_iter().next().unwrap(),
                });
            }
        } else {
            for data in datas {
                if data.lines.0.len() != 0 || data.areas.0.len() != 0 {
                    result.push(Piste {
                        metadata: metadata.clone(),
                        data,
                    });
                }
            }
        }
    }

    result
}

// fn clip_lines(pistes: &mut Vec<Piste>) {
//     for piste in pistes.iter_mut() {
//         piste.data.lines = piste.data.areas.clip(&piste.data.lines, true);
//     }
// }

pub fn parse_pistes(doc: &Document) -> Vec<Piste> {
    let (mut partial_pistes, unnamed_lines, unnamed_areas) =
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

    let mut unnamed_pistes = handle_unnamed_entities(
        unnamed_lines,
        unnamed_areas,
        &mut partial_pistes,
    );
    let mut pistes = merge_empty_refs(create_pistes(partial_pistes));
    if config.is_vv() {
        find_anomalies(&pistes);
    }
    pistes.append(&mut unnamed_pistes);
    // clip_lines(&mut pistes);

    pistes
}
