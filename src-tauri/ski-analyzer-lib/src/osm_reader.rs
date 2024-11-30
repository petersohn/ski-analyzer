use geo::{Coord, Point};
use serde::Deserialize;
use time::OffsetDateTime;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::result::Result as StdResult;

use crate::error::{Error, ErrorType, Result};
use crate::utils::time_ser;

pub type Tags = HashMap<String, String>;

pub fn get_tag<'a>(tags: &'a Tags, name: &str) -> &'a str {
    match tags.get(name) {
        None => "",
        Some(val) => &*val,
    }
}

#[derive(Deserialize, PartialEq, Debug)]
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

#[derive(Deserialize, PartialEq, Debug)]
pub struct Way {
    pub nodes: Vec<u64>,
    #[serde(default)]
    pub tags: Tags,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct RelationMember {
    #[serde(alias = "ref")]
    pub ref_: u64,
    pub role: String,
}

#[derive(Debug, PartialEq)]
pub struct RelationMembers {
    pub nodes: Vec<RelationMember>,
    pub ways: Vec<RelationMember>,
}

impl<'de> Deserialize<'de> for RelationMembers {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "lowercase")]
        enum RelationType {
            Node,
            Way,
        }

        #[derive(Deserialize)]
        struct RelationMemberDef {
            #[serde(alias = "type")]
            type_: RelationType,
            #[serde(flatten)]
            member: RelationMember,
        }

        struct MembersVisitor;

        impl<'de> serde::de::Visitor<'de> for MembersVisitor {
            type Value = RelationMembers;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(formatter, "a list of members")
            }

            fn visit_seq<S>(
                self,
                mut seq: S,
            ) -> StdResult<Self::Value, S::Error>
            where
                S: serde::de::SeqAccess<'de>,
            {
                let mut members = RelationMembers {
                    nodes: Vec::new(),
                    ways: Vec::new(),
                };

                while let Some(member) =
                    seq.next_element::<RelationMemberDef>()?
                {
                    match member.type_ {
                        RelationType::Node => members.nodes.push(member.member),
                        RelationType::Way => members.ways.push(member.member),
                    }
                }

                Ok(members)
            }
        }

        deserializer.deserialize_seq(MembersVisitor)
    }
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Relation {
    pub members: RelationMembers,
    pub tags: Tags,
}

#[derive(Debug, PartialEq, Default)]
pub struct Elements {
    pub nodes: HashMap<u64, Node>,
    pub ways: HashMap<u64, Way>,
    pub relations: HashMap<u64, Relation>,
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

    pub fn get_way<'a>(&'a self, id: &u64) -> Result<&'a Way> {
        match self.ways.get(&id) {
            None => Err(Error::new(
                ErrorType::OSMError,
                format!("way not found: {}", id),
            )),
            Some(val) => Ok(&val),
        }
    }

    pub fn iterate_nodes<'a, 'b, It>(
        &'a self,
        input: It,
    ) -> NodesIterator<'a, 'b, It>
    where
        It: Iterator<Item: Into<&'b u64>>,
    {
        NodesIterator {
            obj: self,
            input,
            phantom_data: PhantomData,
        }
    }
}

pub struct NodesIterator<'a, 'b, It>
where
    It: Iterator<Item: Into<&'b u64>>,
{
    obj: &'a Elements,
    input: It,
    phantom_data: PhantomData<&'b u64>,
}

impl<'a, 'b, It> Iterator for NodesIterator<'a, 'b, It>
where
    It: Iterator<Item: Into<&'b u64>>,
{
    type Item = Result<&'a Node>;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next().map(|id| self.obj.get_node(id.into()))
    }
}

impl<'de> Deserialize<'de> for Elements {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize, Debug)]
        #[serde(tag = "type", rename_all = "lowercase")]
        enum ElementType {
            Node(Node),
            Way(Way),
            Relation(Relation),
        }

        #[derive(Deserialize, Debug)]
        struct Element {
            id: u64,
            #[serde(flatten)]
            type_: ElementType,
        }

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
                    relations: HashMap::new(),
                };

                while let Some(element) = seq.next_element::<Element>()? {
                    match element.type_ {
                        ElementType::Node(node) => {
                            elements.nodes.insert(element.id, node);
                        }
                        ElementType::Way(way) => {
                            elements.ways.insert(element.id, way);
                        }
                        ElementType::Relation(rel) => {
                            elements.relations.insert(element.id, rel);
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
pub struct Osm3s {
    #[serde(with = "time_ser")]
    pub timestamp_osm_base: OffsetDateTime,
    pub copyright: String,
}

#[cfg(test)]
impl Default for Osm3s {
    fn default() -> Self {
        Osm3s {
            timestamp_osm_base: OffsetDateTime::UNIX_EPOCH,
            copyright: String::new(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct Document {
    pub osm3s: Osm3s,
    pub elements: Elements,
}

impl Document {
    pub fn parse(json: &[u8]) -> Result<Self> {
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

pub fn parse_way(doc: &Document, nodes: &Vec<u64>) -> Result<Vec<Coord>> {
    let mut coords: Vec<Coord> = Vec::new();
    coords.reserve(nodes.len());
    for node in doc.elements.iterate_nodes(nodes.iter()) {
        coords.push(node?.into());
    }
    Ok(coords)
}

pub fn parse_ele(tags: &Tags) -> u32 {
    match tags.get("ele") {
        None => 0,
        Some(ele) => ele.parse().unwrap_or(0),
    }
}
