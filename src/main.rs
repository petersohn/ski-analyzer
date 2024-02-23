use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdout, Read, Write};

use curl::easy::Easy;
use url::form_urlencoded;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Element {
    Node {
        id: u64,
        lat: f64,
        lon: f64,
        #[serde(default)]
        tags: HashMap<String, String>,
    },
    Way {
        id: u64,
        nodes: Vec<u64>,
        #[serde(default)]
        tags: HashMap<String, String>,
    },
}

#[derive(Deserialize, Debug)]
struct Document {
    elements: Vec<Element>,
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
    println!("{:?}", doc);

    Ok(())
}
