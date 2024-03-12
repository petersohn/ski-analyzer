use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{stdout, Read, Write};
use std::str::FromStr;

use curl::easy::Easy;
use geo::{Coord, LineString, Point, Polygon};
use strum_macros::EnumString;
use url::form_urlencoded;

#[derive(Debug, Clone)]
struct InvalidInput {
    msg: String,
}

impl InvalidInput {
    fn new(msg: String) -> Self {
        InvalidInput { msg }
    }

    fn new_s(msg: &str) -> Self {
        InvalidInput { msg: msg.into() }
    }

    fn empty() -> Self {
        InvalidInput { msg: String::new() }
    }
}

impl fmt::Display for InvalidInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid OSM input: {}", self.msg)
    }
}

impl Error for InvalidInput {}

type Result_<T> = std::result::Result<T, InvalidInput>;

#[derive(Deserialize, Debug)]
struct Node {
    lat: f64,
    lon: f64,
    #[serde(default)]
    tags: HashMap<String, String>,
}

impl Into<Coord> for &Node {
    fn into(self) -> Coord {
        Coord {
            x: self.lon,
            y: self.lat,
        }
    }
}

impl Into<Point> for &Node {
    fn into(self) -> Point {
        Point::new(self.lon, self.lat)
    }
}

#[derive(Deserialize, Debug)]
struct Way {
    nodes: Vec<u64>,
    #[serde(default)]
    tags: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ElementType {
    Node(Node),
    Way(Way),
}

#[derive(Deserialize, Debug)]
struct Element {
    id: u64,
    #[serde(flatten)]
    type_: ElementType,
}

#[derive(Debug)]
struct Elements {
    nodes: HashMap<u64, Node>,
    ways: HashMap<u64, Way>,
}

impl Elements {
    fn get_node<'a>(&'a self, id: &u64) -> Result_<&'a Node> {
        match self.nodes.get(&id) {
            None => Err(InvalidInput::new(format!("node not found: {}", id))),
            Some(val) => Ok(&val),
        }
    }

    fn iterate_nodes<'a, F>(&'a self, ids: &[u64], mut f: F) -> Result_<()>
    where
        F: FnMut(&'a Node) -> Result_<()>,
    {
        for id in ids {
            f(self.get_node(id)?)?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Elements {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ElementsVisitor;

        impl<'de> serde::de::Visitor<'de> for ElementsVisitor {
            type Value = Elements;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a list of nodes and ways")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: serde::de::SeqAccess<'de>,
            {
                let mut elements = Elements {
                    nodes: HashMap::new(),
                    ways: HashMap::new(),
                };

                while let Some(element) = seq.next_element::<Element>()? {
                    match element.type_ {
                        ElementType::Node(node) => {
                            elements.nodes.insert(element.id, node);
                        }
                        ElementType::Way(way) => {
                            elements.ways.insert(element.id, way);
                        }
                    }
                }

                Ok(elements)
            }
        }

        deserializer.deserialize_seq(ElementsVisitor)
    }
}

#[derive(Deserialize, Debug)]
struct Document {
    elements: Elements,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PointWithElevation {
    point: Point,
    elevation: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Lift {
    name: String,
    type_: String,
    line: LineString,
    begin_altitude: u32,
    end_altitude: u32,
    midstations: Vec<PointWithElevation>,
    can_go_reverse: bool,
    can_disembark: bool,
}

fn get_tag<'a>(tags: &'a HashMap<String, String>, name: &str) -> &'a str {
    match tags.get(name) {
        None => "",
        Some(val) => &*val,
    }
}

fn parse_yesno(value: &str) -> Result_<Option<bool>> {
    match value {
        "" => Ok(None),
        "yes" => Ok(Some(true)),
        "no" => Ok(Some(false)),
        _ => Err(InvalidInput::new(format!("invalid yesno value: {}", value))),
    }
}

impl Lift {
    fn parse(doc: &Document, id: &u64, way: &Way) -> Result_<Option<Self>> {
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

        fn get_access(node: &Node) -> Result_<AccessType> {
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
                        None => ["cable_car", "gondola"].contains(&aerialway_type.as_str()),
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
                    return Err(InvalidInput::new_s("invalid access combination: exit-exit"))
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

        let mut line = parse_way(&doc, &way)?;
        let mut begin_altitude = parse_ele(&begin_node.tags);
        let mut end_altitude = parse_ele(&end_node.tags);

        if reverse {
            line.reverse();
            std::mem::swap(&mut begin_altitude, &mut end_altitude);
        }

        Ok(Some(Lift {
            name,
            type_: aerialway_type.clone(),
            line: LineString::new(line),
            begin_altitude,
            end_altitude,
            midstations,
            can_go_reverse,
            can_disembark: ["drag_lift", "t-bar", "j-bar", "platter", "rope_tow"]
                .contains(&aerialway_type.as_str()),
        }))
    }
}

fn parse_way(doc: &Document, way: &Way) -> Result_<Vec<Coord>> {
    let mut coords: Vec<Coord> = Vec::new();
    coords.reserve(way.nodes.len());
    doc.elements.iterate_nodes(&way.nodes, |node: &Node| {
        coords.push(node.into());
        Ok(())
    })?;
    Ok(coords)
}

fn parse_ele(tags: &HashMap<String, String>) -> u32 {
    match tags.get("ele") {
        None => 0,
        Some(ele) => ele.parse().unwrap_or(0),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SkiArea {
    lifts: Vec<Lift>,
}

impl SkiArea {
    fn parse(doc: &Document) -> Self {
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut input: String =
        form_urlencoded::byte_serialize("[out:json];rel(3545276);>;out;".as_bytes()).collect();
    input.insert_str(0, "data=");
    let mut input_bytes = input.as_bytes();

    let mut easy = Easy::new();
    easy.url("https://overpass-api.de/api/interpreter")?;
    easy.post(true)?;
    easy.post_field_size(input_bytes.len() as u64)?;

    let mut json: Vec<u8> = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.read_function(|buf| Ok(input_bytes.read(buf).unwrap_or(0)))?;
        transfer.write_function(|data| {
            json.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    let doc: Document = serde_json::from_slice(&*json)?;
    // println!("{:#?}", doc);
    // serde_json::to_writer(stdout(), &doc)?;

    let ski_area = SkiArea::parse(&doc);
    println!("{:#?}", ski_area);
    Ok(())
}
