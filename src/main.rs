use std::io::{stdout, Read, Write};
use std::error::Error;

use curl::easy::Easy;
use url::form_urlencoded;

fn main() -> Result<(), Box<dyn Error>> {
    let mut input: String = form_urlencoded::byte_serialize("[out:json];rel(3545276);>;out;".as_bytes()).collect();
    input.insert_str(0, "data=");
    let mut input_bytes = input.as_bytes();

    let mut easy = Easy::new();
    easy.url("https://overpass-api.de/api/interpreter")?;
    easy.post(true)?;
    easy.post_field_size(input_bytes.len() as u64)?;

    let mut transfer = easy.transfer();
    transfer.read_function(|buf| {
        Ok(input_bytes.read(buf).unwrap_or(0))
    })?;
    transfer.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    })?;
    transfer.perform()?;

    Ok(())
}
