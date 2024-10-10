use geo::{
    BooleanOps, BoundingRect, CoordNum, HasDimensions, HaversineLength,
    Intersects, LineString, MultiLineString, MultiPolygon, Polygon, Rect,
};
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use super::UniqueId;
use crate::config::get_config;
// use crate::error::{Error, ErrorType, Result};
use crate::error::Result;
use crate::multipolygon::parse_multipolygon;
use crate::osm_reader::{get_tag, parse_way, Document, Tags, Way};
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::collection::max_if;
use crate::utils::rect::union_rects;

#[derive(
    Serialize,
    Deserialize,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    EnumString,
    strum_macros::Display,
)]
#[strum(serialize_all = "lowercase")]
pub enum Difficulty {
    #[strum(serialize = "")]
    Unknown,
    Novice,
    Easy,
    Intermediate,
    Advanced,
    Expert,
    Freeride,
}

#[derive(PartialEq, Eq, Hash, Clone, Serialize, Deserialize, Debug)]
pub struct PisteMetadata {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub name: String,
    pub difficulty: Difficulty,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PisteData {
    pub bounding_rect: Rect,
    pub areas: MultiPolygon,
    pub lines: MultiLineString,
}

impl geo::Intersects for PisteData {
    fn intersects(&self, other: &PisteData) -> bool {
        self.bounding_rect.intersects(&other.bounding_rect)
            && (self.areas.intersects(&other.areas)
                || self.areas.intersects(&other.lines)
                || self.lines.intersects(&other.areas)
                || self.lines.intersects(&other.lines))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Piste {
    unique_id: String,
    #[serde(flatten)]
    pub metadata: PisteMetadata,
    #[serde(flatten)]
    pub data: PisteData,
}

impl UniqueId for Piste {
    fn get_unique_id(&self) -> &str {
        &self.unique_id
    }
}

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
            if get_config().is_vv() {
                eprintln!(
                    "{} {}: invalid difficulty: {}",
                    name, ref_, difficulty_str
                );
            }
            Difficulty::Unknown
        }
    };

    PisteMetadata {
        ref_: ref_.to_string(),
        name: name.to_string(),
        difficulty,
    }
}

#[derive(Default, Debug)]
struct WithId<T> {
    id: String,
    obj: T,
}

impl<T> WithId<T> {
    fn new(id: String, obj: T) -> Self {
        WithId { id, obj }
    }
}

#[derive(Default, Debug)]
struct PartialPistes {
    line_entities: Vec<WithId<BoundedGeometry<LineString>>>,
    area_entities: Vec<WithId<BoundedGeometry<MultiPolygon>>>,
}

struct UnnamedPiste<T, C = f64>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    difficulty: Difficulty,
    geometry: WithId<BoundedGeometry<T, C>>,
}

enum PisteGeometry {
    Line(BoundedGeometry<LineString>),
    Area(BoundedGeometry<MultiPolygon>),
}

fn add_piste(
    id: String,
    metadata: PisteMetadata,
    geometry: PisteGeometry,
    result: &mut HashMap<PisteMetadata, PartialPistes>,
    unnamed_lines: &mut Vec<UnnamedPiste<LineString>>,
    unnamed_areas: &mut Vec<UnnamedPiste<MultiPolygon>>,
) {
    if metadata.ref_.is_empty() && metadata.name.is_empty() {
        match geometry {
            PisteGeometry::Area(a) => {
                unnamed_areas.push(UnnamedPiste {
                    difficulty: metadata.difficulty,
                    geometry: WithId::new(id, a),
                });
            }
            PisteGeometry::Line(l) => {
                unnamed_lines.push(UnnamedPiste {
                    difficulty: metadata.difficulty,
                    geometry: WithId::new(id, l),
                });
            }
        };
        return;
    }

    let partial_piste = result.entry(metadata).or_default();

    match geometry {
        PisteGeometry::Area(a) => {
            partial_piste.area_entities.push(WithId::new(id, a))
        }
        PisteGeometry::Line(l) => {
            partial_piste.line_entities.push(WithId::new(id, l))
        }
    }
}

fn is_area(way: &Way) -> bool {
    let area = get_tag(&way.tags, "area");
    if area == "yes" {
        return true;
    }

    let is_area = area != "no"
        && way.nodes.len() > 1
        && way.nodes.first() == way.nodes.last();

    if is_area && get_config().is_vv() {
        eprintln!("Implicitly assuming closed line is area: {:?}", way.tags);
    }

    is_area
}

fn parse_partial_piste(doc: &Document, way: &Way) -> Result<PisteGeometry> {
    let coords = parse_way(&doc, &way.nodes)?;
    let line = LineString::new(coords);

    let geometry = if is_area(way) {
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
                metadata.difficulty = md.difficulty;
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
                id.to_string(),
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
                            id.to_string(),
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

fn line_to_piste(
    line: WithId<BoundedGeometry<LineString>>,
) -> WithId<PisteData> {
    WithId::new(
        line.id,
        PisteData {
            bounding_rect: line.obj.bounding_rect,
            areas: MultiPolygon::new(Vec::new()),
            lines: MultiLineString::new(vec![line.obj.item]),
        },
    )
}

fn area_to_piste(
    area: WithId<BoundedGeometry<MultiPolygon>>,
) -> WithId<PisteData> {
    WithId::new(
        area.id,
        PisteData {
            bounding_rect: area.obj.bounding_rect,
            areas: area.obj.item,
            lines: MultiLineString::new(Vec::new()),
        },
    )
}

fn merge_pistes(
    target: &mut WithId<PisteData>,
    source: &mut WithId<PisteData>,
) {
    target.obj.lines.0.append(&mut source.obj.lines.0);
    target.obj.areas.0.append(&mut source.obj.areas.0);
    target.obj.bounding_rect =
        union_rects(target.obj.bounding_rect, source.obj.bounding_rect);
    target.id.push('_');
    target.id.extend([std::mem::take(&mut source.id)]);
}

fn merge_intersecting_pistes(pistes: &mut Vec<WithId<PisteData>>) {
    let mut i = 0;
    while i < pistes.len() - 1 {
        let mut changed = false;
        let mut j = i + 1;
        while j < pistes.len() {
            if pistes[i].obj.intersects(&pistes[j].obj) {
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

fn merge_partial_pistes(
    partial_pistes: HashMap<PisteMetadata, PartialPistes>,
    mut refless: Option<&mut HashMap<PisteMetadata, Vec<WithId<PisteData>>>>,
) -> HashMap<PisteMetadata, Vec<WithId<PisteData>>> {
    let mut result = HashMap::new();
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

        let mut datas: Vec<WithId<PisteData>> = partial_piste
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

        match &mut refless {
            None => merge_intersecting_pistes(&mut datas),
            Some(refless_pistes) => {
                let mut changed = true;
                while changed {
                    changed = false;
                    merge_intersecting_pistes(&mut datas);
                    if let Some(refless_datas) =
                        refless_pistes.get_mut(&PisteMetadata {
                            ref_: String::new(),
                            name: metadata.name.clone(),
                            difficulty: metadata.difficulty,
                        })
                    {
                        for data in datas.iter_mut() {
                            for refless_piste in refless_datas.iter_mut() {
                                if data.obj.intersects(&refless_piste.obj) {
                                    merge_pistes(data, refless_piste);
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        };

        result.insert(metadata, datas);
    }
    result
}

fn make_piste(
    metadata: PisteMetadata,
    data: WithId<PisteData>,
    result: &mut Vec<Piste>,
) {
    if !data.obj.areas.is_empty() || !data.obj.lines.is_empty() {
        result.push(Piste {
            unique_id: data.id,
            metadata,
            data: data.obj,
        });
    }
}

fn make_pistes(
    metadata: PisteMetadata,
    mut datas: Vec<WithId<PisteData>>,
    result: &mut Vec<Piste>,
) {
    match datas.len() {
        0 => (),
        1 => {
            make_piste(metadata, datas.pop().unwrap(), result);
        }
        _ => {
            if get_config().is_vv() {
                eprintln!(
                    "{} {}: piste has {} disjunct parts",
                    metadata.ref_,
                    metadata.name,
                    datas.len()
                );
            }

            for data in datas {
                make_piste(metadata.clone(), data, result);
            }
        }
    }
}

fn create_pistes(
    partial_pistes: HashMap<PisteMetadata, PartialPistes>,
) -> Vec<Piste> {
    let mut refless: HashMap<PisteMetadata, PartialPistes> = HashMap::new();
    let mut reffed: HashMap<PisteMetadata, PartialPistes> = HashMap::new();
    for (metadata, piste) in partial_pistes {
        if metadata.ref_.is_empty() {
            refless.insert(metadata, piste);
        } else {
            reffed.insert(metadata, piste);
        }
    }

    let mut refless_datas = merge_partial_pistes(refless, None);
    let reffed_datas = merge_partial_pistes(reffed, Some(&mut refless_datas));

    let mut result = Vec::new();
    result.reserve(reffed_datas.len() + refless_datas.len());

    for (metadata, datas) in refless_datas.into_iter().chain(reffed_datas) {
        make_pistes(metadata, datas, &mut result);
    }

    result
}

fn merge_unnamed_pistes(
    unnamed_lines: Vec<UnnamedPiste<LineString>>,
    unnamed_areas: Vec<UnnamedPiste<MultiPolygon>>,
) -> Vec<Piste> {
    let mut pistes: HashMap<Difficulty, Vec<WithId<PisteData>>> =
        HashMap::new();
    for line in unnamed_lines {
        pistes
            .entry(line.difficulty)
            .or_default()
            .push(line_to_piste(line.geometry));
    }
    for area in unnamed_areas {
        pistes
            .entry(area.difficulty)
            .or_default()
            .push(area_to_piste(area.geometry));
    }

    for mut datas in pistes.values_mut() {
        merge_intersecting_pistes(&mut datas);
    }

    let it = pistes.into_iter().map(|(difficulty, datas)| {
        datas.into_iter().map(move |data| Piste {
            unique_id: data.id,
            metadata: PisteMetadata {
                ref_: String::new(),
                name: String::new(),
                difficulty,
            },
            data: data.obj,
        })
    });
    it.flatten().collect()
}

fn merge_unnamed_entities<G1, G2, GetEntity, GetOtherEntity, GetLength>(
    mut input: Vec<UnnamedPiste<G1>>,
    partial_pistes: &mut HashMap<PisteMetadata, PartialPistes>,
    get_entity: GetEntity,
    get_other_entity: GetOtherEntity,
    get_length: GetLength,
) -> (Vec<UnnamedPiste<G1>>, bool)
where
    G1: BoundingRect<f64>,
    G2: BoundingRect<f64>,
    GetEntity: Fn(&mut PartialPistes) -> &mut Vec<WithId<BoundedGeometry<G1>>>,
    GetOtherEntity: Fn(&PartialPistes) -> &Vec<WithId<BoundedGeometry<G2>>>,
    GetLength: Fn(&BoundedGeometry<G1>, &BoundedGeometry<G2>) -> f64,
{
    let mut rest = Vec::new();
    let mut changed = false;

    while let Some(g1) = input.pop() {
        let target = max_if(
            partial_pistes.iter_mut(),
            |piste| {
                get_other_entity(piste.1).iter().fold(0.0, |acc, g2| {
                    acc + get_length(&g1.geometry.obj, &g2.obj)
                })
            },
            |piste, len| *len > 0.0 && piste.0.difficulty == g1.difficulty,
        );
        match target {
            Some((_, piste)) => {
                get_entity(piste).push(g1.geometry);
                changed = true;
            }
            None => rest.push(g1),
        }
    }

    (rest, changed)
}

fn handle_unnamed_entities(
    mut unnamed_lines: Vec<UnnamedPiste<LineString>>,
    mut unnamed_areas: Vec<UnnamedPiste<MultiPolygon>>,
    partial_pistes: &mut HashMap<PisteMetadata, PartialPistes>,
) -> Vec<Piste> {
    loop {
        let (unnamed_areas2, changed1) = merge_unnamed_entities(
            unnamed_areas,
            partial_pistes,
            |p| &mut p.area_entities,
            |p| &p.line_entities,
            |a, l| get_intersection_length(a, l),
        );
        let (unnamed_lines2, changed2) = merge_unnamed_entities(
            unnamed_lines,
            partial_pistes,
            |p| &mut p.line_entities,
            |p| &p.area_entities,
            |l, a| get_intersection_length(a, l),
        );

        unnamed_areas = unnamed_areas2.into();
        unnamed_lines = unnamed_lines2.into();
        if !changed1 && !changed2 {
            break;
        }
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
    let mut pistes = create_pistes(partial_pistes);
    if config.is_vv() {
        find_anomalies(&pistes);
    }
    pistes.append(&mut unnamed_pistes);
    // clip_lines(&mut pistes);

    pistes
}
