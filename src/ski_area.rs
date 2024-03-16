use geo::{BoundingRect, Coord, CoordNum, LineString, Point, Polygon, Rect};
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
pub struct BoundedGeometry<T, C = f64>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    pub item: T,
    pub bounding_rect: Rect<C>,
}

impl<T, C> BoundedGeometry<T, C>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    pub fn new(item: T) -> Result<Self> {
        let bounding_rect = item
            .bounding_rect()
            .into()
            .ok_or(InvalidInput::new_s("cannot calculate bounding rect"))?;
        Ok(BoundedGeometry {
            item,
            bounding_rect,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lift {
    pub name: String,
    pub type_: String,
    pub line: BoundedGeometry<LineString>,
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

// TODO: handle funiculars
impl Lift {
    fn parse(doc: &Document, id: &u64, way: &Way) -> Result<Option<Self>> {
        if get_tag(&way.tags, "area") == "yes" {
            return Ok(None);
        }

        let Some(aerialway_type) = way.tags.get("aerialway") else {
            return Ok(None);
        };

        let allowed_types = [
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
        let ignored_types = ["goods", "pylon", "station", "construction"];
        if ignored_types.contains(&aerialway_type.as_str()) {
            return Ok(None);
        }
        if !allowed_types.contains(&aerialway_type.as_str()) {
            return Err(InvalidInput::new(format!(
                "invalid lift type: {}",
                aerialway_type
            )));
        }

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

        let mut name = get_tag(&way.tags, "name").to_string();

        if name == "" {
            eprintln!("{}: {} lift has no name", id, aerialway_type);
            name = format!("<unnamed {}>", aerialway_type);
        }

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

        let line = BoundedGeometry::new(LineString::new(line_points))?;
        let can_disembark =
            ["drag_lift", "t-bar", "j-bar", "platter", "rope_tow"]
                .contains(&aerialway_type.as_str());

        Ok(Some(Lift {
            name,
            type_: aerialway_type.clone(),
            line,
            begin_altitude,
            end_altitude,
            midstations,
            can_go_reverse,
            can_disembark,
        }))
    }
}

// #[derive(
//     Serialize,
//     Deserialize,
//     Debug,
//     PartialEq,
//     Eq,
//     EnumString,
//     strum_macros::Display,
// )]
// #[strum(serialize_all = "lowercase")]
// pub enum Difficulty {
//     Novice,
//     Easy,
//     Intermediate,
//     Advanced,
//     Expert,
//     Freeride,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// pub struct Piste {}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkiArea {
    name: String,
    lifts: Vec<Lift>,
}

impl SkiArea {
    pub fn parse(doc: &Document) -> Result<Self> {
        let mut names: Vec<String> = Vec::new();
        let mut lifts = Vec::new();
        for (id, way) in &doc.elements.ways {
            if get_tag(&way.tags, "landuse") == "winter_sports" {
                names.push(get_tag(&way.tags, "name").to_string());
                continue;
            }

            match Lift::parse(&doc, &id, &way) {
                Err(e) => eprintln!("Error parsing way {}: {}", id, e),
                Ok(None) => (),
                Ok(Some(lift)) => lifts.push(lift),
            }
        }
        eprintln!("Found {} lifts.", lifts.len());

        if names.len() == 0 {
            Err(InvalidInput::new_s("ski area entity not found"))
        } else if names.len() > 1 {
            Err(InvalidInput::new(format!("ambiguous ski area: {:?}", names)))
        } else {
            Ok(SkiArea { name: names.remove(0), lifts })
        }
    }
}
