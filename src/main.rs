use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdout, Read, Write};

use curl::easy::Easy;
use geo::{LineString, Polygon};
use url::form_urlencoded;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ElementType {
    Node { lat: f64, lon: f64 },
    Way { nodes: Vec<u64> },
}

#[derive(Deserialize, Debug)]
struct Element {
    id: u64,
    #[serde(default)]
    tags: HashMap<String, String>,
    #[serde(flatten)]
    type_: ElementType,
}

#[derive(Deserialize, Debug)]
struct Document {
    elements: Vec<Element>,
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
    println!("{:?}", doc);

    Ok(())
}
