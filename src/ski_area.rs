use geo::{BoundingRect, Coord, LineString, Point, Polygon, Rect};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::EnumString;

use crate::error::{InvalidInput, Result};
use crate::osm_reader::{get_tag, Document, Node, Tags, Way};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointWithElevation {
    point: Point,
    elevation: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lift {
    pub name: String,
    pub type_: String,
    pub line: LineString,
    pub bounding_rect: Rect,
    pub begin_altitude: u32,
    pub end_altitude: u32,
    pub midstations: Vec<PointWithElevation>,
    pub can_go_reverse: bool,
    pub can_disembark: bool,
}

fn parse_yesno(value: &str) -> Result<Option<bool>> {
    match value {
        "" => Ok(None),
        "yes" => Ok(Some(true)),
        "no" => Ok(Some(false)),
        _ => Err(InvalidInput::new(format!("invalid yesno value: {}", value))),
    }
}

impl Lift {
    fn parse(doc: &Document, id: &u64, way: &Way) -> Result<Option<Self>> {
        let Some(aerialway_type) = way.tags.get("aerialway") else {
            return Ok(None);
        };

        let Some((begin_id, rest)) = way.nodes.split_first() else {
            return Err(InvalidInput::new_s("empty lift"));
        };
        let Some((end_id, midpoints)) = rest.split_last() else {
            return Err(InvalidInput::new_s("lift has a single point"));
        };

        fn is_station(node: &Node) -> bool {
            get_tag(&node.tags, "aerialway") == "station"
        }

        let mut midstations = Vec::new();
        let mut midstation_nodes: Vec<&Node> = Vec::new();
        doc.elements.iterate_nodes(&midpoints, |node: &Node| {
            if is_station(&node) {
                midstations.push(PointWithElevation {
                    point: node.into(),
                    elevation: parse_ele(&node.tags),
                });
                midstation_nodes.push(&node);
            }
            Ok(())
        })?;

        #[derive(PartialEq, Eq, EnumString, strum_macros::Display)]
        #[strum(serialize_all = "lowercase")]
        enum AccessType {
            #[strum(serialize = "")]
            Unknown,
            Entry,
            Exit,
            Both,
        }

        fn get_access(node: &Node) -> Result<AccessType> {
            if !is_station(&node) {
                return Ok(AccessType::Unknown);
            }

            let access = get_tag(&node.tags, "aerialway:access");
            AccessType::from_str(&access).or(Err(InvalidInput::new(format!(
                "invalid access type: {}",
                access
            ))))
        }

        let begin_node = doc.elements.get_node(begin_id)?;
        let begin_access = get_access(&begin_node)?;
        let end_node = doc.elements.get_node(end_id)?;
        let end_access = get_access(&end_node)?;

        let name = get_tag(&way.tags, "name").into();

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
                    return Err(InvalidInput::new_s(
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
                    return Err(InvalidInput::new_s(
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

        if is_unusual {
            let mut accesses: Vec<&str> = Vec::new();
            accesses.reserve(midstation_nodes.len() + 2);
            let begin_access_s = begin_access.to_string();
            accesses.push(&begin_access_s);
            for node in midstation_nodes {
                accesses.push(get_tag(&node.tags, "aerialway:access"));
            }
            let end_access_s = end_access.to_string();
            accesses.push(&end_access_s);
            eprintln!(
                "{} {}: Unusual station combination: {:?}",
                id, name, accesses
            )
        }

        if let Some(oneway_) = oneway {
            let actual_can_go_reverse = !oneway_;
            if actual_can_go_reverse != can_go_reverse {
                eprintln!(
                    "{} {}: lift can_go_reverse mismatch: calculated={}, actual={}",
                    id, name, can_go_reverse, actual_can_go_reverse
                );
                can_go_reverse = actual_can_go_reverse;
            }
        }

        let mut line_points = parse_way(&doc, &way)?;
        let mut begin_altitude = parse_ele(&begin_node.tags);
        let mut end_altitude = parse_ele(&end_node.tags);

        if reverse {
            eprintln!("{} {}: lift goes in reverse", id, name);
            line_points.reverse();
            std::mem::swap(&mut begin_altitude, &mut end_altitude);
        }

        let line = LineString::new(line_points);
        let bounding_rect = line
            .bounding_rect()
            .ok_or(InvalidInput::new_s("cannot calculate bounding rect"))?;
        let can_disembark =
            ["drag_lift", "t-bar", "j-bar", "platter", "rope_tow"]
                .contains(&aerialway_type.as_str());

        Ok(Some(Lift {
            name,
            type_: aerialway_type.clone(),
            line,
            bounding_rect,
            begin_altitude,
            end_altitude,
            midstations,
            can_go_reverse,
            can_disembark,
        }))
    }
}

fn parse_way(doc: &Document, way: &Way) -> Result<Vec<Coord>> {
    let mut coords: Vec<Coord> = Vec::new();
    coords.reserve(way.nodes.len());
    doc.elements.iterate_nodes(&way.nodes, |node: &Node| {
        coords.push(node.into());
        Ok(())
    })?;
    Ok(coords)
}

fn parse_ele(tags: &Tags) -> u32 {
    match tags.get("ele") {
        None => 0,
        Some(ele) => ele.parse().unwrap_or(0),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkiArea {
    lifts: Vec<Lift>,
}

impl SkiArea {
    pub fn parse(doc: &Document) -> Self {
        let mut lifts = Vec::new();
        for (id, way) in &doc.elements.ways {
            match Lift::parse(&doc, &id, &way) {
                Err(e) => eprintln!("Error parsing way {}: {}", id, e),
                Ok(None) => (),
                Ok(Some(lift)) => lifts.push(lift),
            }
        }
        eprintln!("Found {} lifts.", lifts.len());
        SkiArea { lifts }
    }
}
