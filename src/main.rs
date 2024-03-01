use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdout, Read, Write};

use curl::easy::Easy;
use geo::{LineString, Polygon};
use url::form_urlencoded;

#[derive(Deserialize, Debug)]
struct Node {
    lat: f64,
    lon: f64,
    #[serde(default)]
    tags: HashMap<String, String>,
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

// #[derive(Serialize, Deserialize, Debug)]
// struct Lift {
//     name: String,
//     line: LineString,
//     bottom_altitude: f64,
//     top_altitude: f64,
// }
//
// struct SkiArea {
//     lifts: Vec<Lift>,
// }
//
// impl From<Document> for SkiArea {
//     fn from(document: Document) -> Self {
//     }
// }

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
