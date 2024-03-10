use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{stdout, Read, Write};

use curl::easy::Easy;
use geo::{Coord, LineString, Point, Polygon};
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
        write!(f, "invalid OSM input")
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
    fn get_node(&self, id: &u64) -> Result_<&Node> {
        match self.nodes.get(&id) {
            None => Err(InvalidInput::new(format!("node not found: {}", id))),
            Some(val) => Ok(&val),
        }
    }

    fn iterate_nodes<F>(&self, ids: &[u64], mut f: F) -> Result_<()>
    where
        F: FnMut(&Node) -> Result_<()>,
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
    line: LineString,
    begin_altitude: u32,
    end_altitude: u32,
    midstations: Vec<PointWithElevation>,
    can_go_reverse: bool,
    can_disembark: bool,
}

impl Lift {
    fn parse(doc: &Document, way: &Way) -> Result_<Option<Self>> {
        let Some(aerialway_type) = way.tags.get("aerialway") else {
            return Ok(None);
        };

        let Some((first, rest)) = way.nodes.split_first() else {
            return Err(InvalidInput::new_s("empty lift"));
        };
        let Some((last, midpoints)) = rest.split_last() else {
            return Err(InvalidInput::new_s("lift has a single point"));
        };
        let mut midstations: Vec<PointWithElevation> = Vec::new();
        doc.elements.iterate_nodes(&midpoints, |node: &Node| {
            if let Some(t) = node.tags.get("aerialway") {
                if t == "station" {
                    midstations.push(PointWithElevation {
                        point: node.into(),
                        elevation: parse_ele(&node.tags),
                    });
                }
            }
            Ok(())
        })?;

        Ok(Some(Lift {
            name: way.tags.get("name").unwrap_or(&String::new()).clone(),
            line: parse_way(&doc, &way)?,
            begin_altitude: parse_ele(
                &doc.elements
                    .get_node(way.nodes.first().ok_or(InvalidInput::empty())?)?
                    .tags,
            ),
            end_altitude: parse_ele(
                &doc.elements
                    .get_node(way.nodes.last().ok_or(InvalidInput::empty())?)?
                    .tags,
            ),
            midstations,
            can_go_reverse: false,
            can_disembark: false,
        }))
    }
}

fn parse_way(doc: &Document, way: &Way) -> Result_<LineString> {
    let mut coords: Vec<Coord> = Vec::new();
    coords.reserve(way.nodes.len());
    doc.elements.iterate_nodes(&way.nodes, |node: &Node| {
        coords.push(node.into());
        Ok(())
    })?;
    Ok(LineString::new(coords))
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
    fn new() -> Self {
        SkiArea { lifts: Vec::new() }
    }
}

impl From<&Document> for SkiArea {
    fn from(document: &Document) -> Self {
        let mut result = SkiArea::new();
        for (_id, way) in document.elements.ways.iter() {}
        result
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
    println!("{:#?}", doc);
    // serde_json::to_writer(stdout(), &doc)?;

    Ok(())
}
