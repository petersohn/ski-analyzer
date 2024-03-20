use crate::error::{Error, ErrorType, Result};
use curl::easy::Easy;
use std::io::Read;
use std::result::Result as StdResult;
use url::form_urlencoded;

fn query_inner(query: &str) -> StdResult<Vec<u8>, curl::Error> {
    let mut input: String =
        form_urlencoded::byte_serialize(&query.as_bytes()).collect();
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
    Ok(json)
}

pub fn query(query: &str) -> Result<Vec<u8>> {
    Ok(query_inner(&query).or_else(|err| {
        Err(Error::convert(
            ErrorType::ExternalError,
            "query error",
            &err,
        ))
    })?)
}

pub fn query_ski_area(name: &str) -> Result<Vec<u8>> {
    let query_string = String::from(
        r###"[out:json];(way[landuse="winter_sports"][name~""###,
    ) + name
        + r###"",i]->.a;(way(area.a)["aerialway"];way(area.a)["piste:type"="downhill"];);node(w););out;"###;
    query(query_string.as_str())
}
