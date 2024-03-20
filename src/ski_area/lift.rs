use geo::LineString;
use strum_macros::EnumString;

use std::str::FromStr;

use super::{BoundedGeometry, Lift, PointWithElevation};

use crate::config::get_config;
use crate::error::{Error, ErrorType, Result};
use crate::osm_reader::{
    get_tag, parse_ele, parse_way, parse_yesno, Document, Node, Way,
};

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

fn get_access(node: &Node) -> Result<AccessType> {
    if !is_station(&node) {
        return Ok(AccessType::Unknown);
    }

    let access = get_tag(&node.tags, "aerialway:access");
    AccessType::from_str(&access).or(Err(Error::new(
        ErrorType::OSMError,
        format!("invalid access type: {}", access),
    )))
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
pub fn parse_lift(doc: &Document, id: &u64, way: &Way) -> Result<Option<Lift>> {
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

    let Some((begin_id, rest)) = way.nodes.split_first() else {
        return Err(Error::new_s(ErrorType::OSMError, "empty lift"));
    };
    let Some((end_id, midpoints)) = rest.split_last() else {
        return Err(Error::new_s(
            ErrorType::OSMError,
            "lift has a single point",
        ));
    };

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

    let begin_node = doc.elements.get_node(begin_id)?;
    let begin_access = get_access(&begin_node)?;
    let end_node = doc.elements.get_node(end_id)?;
    let end_access = get_access(&end_node)?;

    let mut name = get_tag(&way.tags, "name").to_string();
    let ref_ = get_tag(&way.tags, "ref").to_string();

    let config = get_config();

    if name == "" {
        if config.verbose {
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

    if is_unusual && config.verbose {
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
            id, ref_name, accesses
        )
    }

    if let Some(oneway_) = oneway {
        let actual_can_go_reverse = !oneway_;
        if actual_can_go_reverse != can_go_reverse {
            if config.verbose {
                eprintln!(
                        "{} {}: lift can_go_reverse mismatch: calculated={}, actual={}",
                        id, name, can_go_reverse, actual_can_go_reverse
                    );
            }
            can_go_reverse = actual_can_go_reverse;
        }
    }

    let mut line_points = parse_way(&doc, &way)?;
    let mut begin_altitude = parse_ele(&begin_node.tags);
    let mut end_altitude = parse_ele(&end_node.tags);

    if reverse {
        if config.verbose {
            eprintln!("{} {}: lift goes in reverse", id, ref_name);
        }
        line_points.reverse();
        std::mem::swap(&mut begin_altitude, &mut end_altitude);
    }

    let line = BoundedGeometry::new(LineString::new(line_points))?;
    let can_disembark = DRAGLIFT_TYPES.contains(&aerialway_type.as_str());

    Ok(Some(Lift {
        ref_,
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
