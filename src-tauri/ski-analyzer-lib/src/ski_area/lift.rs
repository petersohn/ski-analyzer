use geo::{
    Closest, Distance, Haversine, HaversineClosestPoint, Length, Line,
    LineString, Point,
};
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use std::str::FromStr;

use super::PointWithElevation;
use crate::config::{self, get_config};
use crate::error::{Error, ErrorType, Result};
use crate::osm_reader::{
    get_tag, parse_ele, parse_way, parse_yesno, Document, Node, Way,
};
use crate::utils::bounded_geometry::BoundedGeometry;

pub struct LiftClosestPoint {
    pub line_id: usize,
    pub line: Line,
    pub point: Point,
    pub distance: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lift {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub line: BoundedGeometry<LineString>,
    pub stations: Vec<PointWithElevation>,
    pub can_go_reverse: bool,
    pub can_disembark: bool,
    pub lengths: Vec<f64>,
}

impl Lift {
    pub fn get_closest_point(&self, p: Point) -> Option<LiftClosestPoint> {
        self.line
            .item
            .lines()
            .enumerate()
            .map(|(line_id, line)| {
                let point = match line.haversine_closest_point(&p) {
                    Closest::Intersection(p2) => p2,
                    Closest::SinglePoint(p2) => p2,
                    Closest::Indeterminate => {
                        panic!(
                            "Cannot determine distance between {:?} and {:?}",
                            p, line
                        );
                    }
                };
                LiftClosestPoint {
                    line_id,
                    line,
                    point,
                    distance: Haversine::distance(p, point),
                }
            })
            .min_by(|d1, d2| d1.distance.total_cmp(&d2.distance))
    }
}

impl PartialEq for Lift {
    fn eq(&self, other: &Self) -> bool {
        self.ref_ == other.ref_
            && self.name == other.name
            && self.type_ == other.type_
            && self.line == other.line
            && self.can_go_reverse == other.can_go_reverse
            && self.can_disembark == other.can_disembark
    }
}

#[derive(PartialEq, Eq, EnumString, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
enum AccessType {
    #[strum(serialize = "")]
    Unknown,
    Entry,
    Exit,
    Both,
}

fn is_station(node: &Node) -> bool {
    get_tag(&node.tags, "aerialway") == "station"
}

fn get_access(node: &Node) -> AccessType {
    if !is_station(&node) {
        return AccessType::Unknown;
    }

    let access = get_tag(&node.tags, "aerialway:access");
    AccessType::from_str(&access).unwrap_or_else(|e| {
        if get_config().is_v() {
            eprintln!("{}", e);
        }
        AccessType::Unknown
    })
}

const ALLOWED_TYPES: &[&str] = &[
    "cable_car",
    "gondola",
    "mixed_lift",
    "chair_lift",
    "drag_lift",
    "t-bar",
    "j-bar",
    "platter",
    "rope_tow",
    "magic_carpet",
    "zip_line",
];
const IGNORED_TYPES: &[&str] =
    &["goods", "pylon", "station", "construction", "yes"];

const DRAGLIFT_TYPES: &[&str] =
    &["drag_lift", "t-bar", "j-bar", "platter", "rope_tow"];

// TODO: handle funiculars
pub fn parse_lift<'d>(
    doc: &'d Document,
    id: &u64,
    way: &Way,
) -> Result<Option<Lift>> {
    if get_tag(&way.tags, "area") == "yes" {
        return Ok(None);
    }

    let Some(aerialway_type) = way.tags.get("aerialway") else {
        return Ok(None);
    };

    if IGNORED_TYPES.contains(&aerialway_type.as_str()) {
        return Ok(None);
    }
    if !ALLOWED_TYPES.contains(&aerialway_type.as_str()) {
        return Err(Error::new(
            ErrorType::OSMError,
            format!("invalid lift type: {}", aerialway_type),
        ));
    }

    if way.nodes.len() < 2 {
        return Err(Error::new_s(
            ErrorType::OSMError,
            "Lift doesn't have enought points",
        ));
    }

    let (begin_id, rest) = way.nodes.split_first().unwrap();
    let (end_id, midpoints) = rest.split_last().unwrap();
    let begin_node = doc.elements.get_node(begin_id)?;
    let begin_access = get_access(&begin_node);
    let end_node = doc.elements.get_node(end_id)?;
    let end_access = get_access(&end_node);

    #[derive(Debug)]
    struct StationInfo<'a> {
        station: PointWithElevation,
        node: &'a Node,
        num: usize,
    }

    #[derive(Debug, Default)]
    struct StationInfos<'a>(Vec<StationInfo<'a>>);

    impl<'a> StationInfos<'a> {
        fn add(&mut self, num: usize, node: &'a Node) {
            self.0.push(StationInfo {
                station: PointWithElevation::new(
                    node.coordinate.to_point(),
                    parse_ele(&node.tags),
                ),
                node,
                num,
            });
        }
    }

    let mut station_infos = StationInfos::default();
    let config = get_config();

    station_infos.add(0, begin_node);
    doc.elements
        .iterate_nodes(midpoints.iter())
        .enumerate()
        .filter(|(_, r)| r.as_ref().map_or(true, |n| is_station(n)))
        .try_for_each(|(i, n)| Ok(station_infos.add(i + 1, n?)))?;
    station_infos.add(way.nodes.len() - 1, end_node);

    let mut name = get_tag(&way.tags, "name").to_string();
    let ref_ = get_tag(&way.tags, "ref").to_string();

    if name == "" {
        if config.is_vv() {
            eprintln!("{} {}: {} lift has no name", id, ref_, aerialway_type);
        }
        name = if ref_ == "" {
            format!("<unnamed {}>", aerialway_type)
        } else {
            ref_.clone()
        };
    }

    let ref_name = if ref_ == "" {
        name.clone()
    } else {
        format!("{} ({})", name, ref_)
    };

    let oneway = parse_yesno(&get_tag(&way.tags, "oneway"))?;

    let (reverse, mut can_go_reverse, is_unusual) = match begin_access {
        AccessType::Unknown => match end_access {
            AccessType::Unknown => {
                let can_go_reverse = match oneway {
                    Some(val) => !val,
                    None => ["cable_car", "gondola"]
                        .contains(&aerialway_type.as_str()),
                };
                (false, can_go_reverse, false)
            }
            AccessType::Entry => (true, false, true),
            AccessType::Exit => (false, false, true),
            AccessType::Both => (false, true, true),
        },
        AccessType::Entry => match end_access {
            AccessType::Unknown => (false, false, true),
            AccessType::Entry => {
                return Err(Error::new_s(
                    ErrorType::OSMError,
                    "invalid access combination: entry-entry",
                ))
            }
            AccessType::Exit => (false, false, false),
            AccessType::Both => (false, false, true),
        },
        AccessType::Exit => match end_access {
            AccessType::Unknown => (true, false, true),
            AccessType::Entry => (true, false, false),
            AccessType::Exit => {
                return Err(Error::new_s(
                    ErrorType::OSMError,
                    "invalid access combination: exit-exit",
                ))
            }
            AccessType::Both => (true, false, true),
        },
        AccessType::Both => match end_access {
            AccessType::Unknown => (false, true, true),
            AccessType::Entry => (true, false, true),
            AccessType::Exit => (false, false, true),
            AccessType::Both => (false, true, false),
        },
    };

    if is_unusual && config.is_vv() {
        let accesses: Vec<String> = station_infos
            .0
            .iter()
            .map(|s| get_access(&s.node).to_string())
            .collect();
        eprintln!(
            "{} {}: Unusual station combination: {:?}",
            id, ref_name, accesses
        )
    }

    if let Some(oneway_) = oneway {
        let actual_can_go_reverse = !oneway_;
        if actual_can_go_reverse != can_go_reverse {
            if config.is_vv() {
                eprintln!(
                        "{} {}: lift can_go_reverse mismatch: calculated={}, actual={}",
                        id, name, can_go_reverse, actual_can_go_reverse
                    );
            }
            can_go_reverse = actual_can_go_reverse;
        }
    }

    let mut line_points = parse_way(&doc, &way.nodes)?;
    let mut lengths: Vec<f64> = station_infos
        .0
        .windows(2)
        .into_iter()
        .map(|ss| {
            line_points[ss[0].num..(ss[1].num + 1)]
                .windows(2)
                .map(|ps| Line::new(ps[0], ps[1]).length::<Haversine>())
                .sum()
        })
        .collect();
    let mut stations: Vec<PointWithElevation> =
        station_infos.0.into_iter().map(|s| s.station).collect();

    if reverse {
        if config.is_vv() {
            eprintln!("{} {}: lift goes in reverse", id, ref_name);
        }
        line_points.reverse();
        stations.reverse();
        lengths.reverse();
    }

    let line = BoundedGeometry::new(LineString::new(line_points))?;
    let can_disembark = DRAGLIFT_TYPES.contains(&aerialway_type.as_str());

    Ok(Some(Lift {
        ref_,
        name,
        type_: aerialway_type.clone(),
        line,
        stations,
        can_go_reverse,
        can_disembark,
        lengths,
    }))
}
