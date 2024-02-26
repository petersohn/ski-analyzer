use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdout, Read, Write};

use curl::easy::Easy;
use geo::{LineString, Polygon};
use url::form_urlencoded;

#[derive(Serialize, Deserialize, Debug)]
struct Node {
    lat: f64,
    lon: f64,
    #[serde(default)]
    tags: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
struct DocumentBase {
    elements: Vec<Element>,
}

#[derive(Serialize, Debug)]
struct Document {
    nodes: HashMap<u64, Node>,
    ways: HashMap<u64, Way>,
}

impl From<DocumentBase> for Document {
    fn from(input: DocumentBase) -> Document {
        let mut result = Document{ nodes : HashMap::new(), ways : HashMap::new() };
        for element in input.elements.into_iter() {
            match element.type_ {
                ElementType::Node(node) => {
                    result.nodes.insert(element.id, node);
                    ()
                }
                ElementType::Way(way) => {
                    result.ways.insert(element.id, way);
                    ()
                }
            }
        }
        result
    }
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
    let doc_: DocumentBase = serde_json::from_slice(&*json)?;
    let doc = Document::from(doc_);
    serde_json::to_writer(stdout(), &doc)?;

    Ok(())
}
