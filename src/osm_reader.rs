use geo::{Coord, Point};
use serde::Deserialize;
use std::collections::HashMap;
use std::result::Result as StdResult;

use crate::error::{Error, ErrorType, Result};

pub type Tags = HashMap<String, String>;

pub fn get_tag<'a>(tags: &'a Tags, name: &str) -> &'a str {
    match tags.get(name) {
        None => "",
        Some(val) => &*val,
    }
}

#[derive(Deserialize, Debug)]
pub struct Node {
    pub lat: f64,
    pub lon: f64,
    #[serde(default)]
    pub tags: Tags,
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
pub struct Way {
    pub nodes: Vec<u64>,
    #[serde(default)]
    pub tags: Tags,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ElementType {
    Node(Node),
    Way(Way),
}

#[derive(Deserialize, Debug)]
pub struct Element {
    pub id: u64,
    #[serde(flatten)]
    pub type_: ElementType,
}

#[derive(Debug)]
pub struct Elements {
    pub nodes: HashMap<u64, Node>,
    pub ways: HashMap<u64, Way>,
}

impl Elements {
    pub fn get_node<'a>(&'a self, id: &u64) -> Result<&'a Node> {
        match self.nodes.get(&id) {
            None => Err(Error::new(
                ErrorType::OSMError,
                format!("node not found: {}", id),
            )),
            Some(val) => Ok(&val),
        }
    }

    pub fn iterate_nodes<'a, F>(&'a self, ids: &[u64], mut f: F) -> Result<()>
    where
        F: FnMut(&'a Node) -> Result<()>,
    {
        for id in ids {
            f(self.get_node(id)?)?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Elements {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ElementsVisitor;

        impl<'de> serde::de::Visitor<'de> for ElementsVisitor {
            type Value = Elements;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(formatter, "a list of nodes and ways")
            }

            fn visit_seq<S>(
                self,
                mut seq: S,
            ) -> StdResult<Self::Value, S::Error>
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
pub struct Document {
    pub elements: Elements,
}

impl Document {
    pub fn parse(json: &Vec<u8>) -> Result<Self> {
        let doc: Document = serde_json::from_slice(&*json).or_else(|err| {
            Err(Error::convert(
                ErrorType::OSMError,
                "JSON decode error",
                &err,
            ))
        })?;
        Ok(doc)
    }
}

pub fn parse_yesno(value: &str) -> Result<Option<bool>> {
    match value {
        "" => Ok(None),
        "yes" => Ok(Some(true)),
        "no" => Ok(Some(false)),
        _ => Err(Error::new(
            ErrorType::OSMError,
            format!("invalid yesno value: {}", value),
        )),
    }
}

pub fn parse_way(doc: &Document, way: &Way) -> Result<Vec<Coord>> {
    let mut coords: Vec<Coord> = Vec::new();
    coords.reserve(way.nodes.len());
    doc.elements.iterate_nodes(&way.nodes, |node: &Node| {
        coords.push(node.into());
        Ok(())
    })?;
    Ok(coords)
}

pub fn parse_ele(tags: &Tags) -> u32 {
    match tags.get("ele") {
        None => 0,
        Some(ele) => ele.parse().unwrap_or(0),
    }
}
