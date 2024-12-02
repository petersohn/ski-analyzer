use crate::config::get_config;
use crate::error::{Error, ErrorType, Result};
use curl::easy::Easy;
use geo::Rect;
use std::io::Read;
use std::result::Result as StdResult;
use url::form_urlencoded;

fn query_inner(query: &str) -> StdResult<Vec<u8>, curl::Error> {
    let mut input: String =
        form_urlencoded::byte_serialize(&query.as_bytes()).collect();
    input.insert_str(0, "data=");

    if get_config().is_v() {
        eprintln!("{}", input);
    }

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

pub fn query_ski_area_details_by_id(id: u64) -> Result<Vec<u8>> {
    let query_string = format!(
        r###"[out:json];
(
    way({})->.a;
    (
        way(area.a)["aerialway"];
        way(area.a)["piste:type"="downhill"];
        rel(area.a)["piste:type"="downhill"];
    );
    >;
);
out;"###,
        id
    );
    query(query_string.as_str())
}

pub fn query_ski_areas_by_name(name: &str) -> Result<Vec<u8>> {
    let query_string = format!(
        r###"[out:json];
way[landuse="winter_sports"][name~"{}",i];
out geom;"###,
        name
    );
    query(query_string.as_str())
}

pub fn query_ski_areas_by_coords(rect: Rect) -> Result<Vec<u8>> {
    let query_string = format!(
        r###"[out:json];
is_in({n}, {w})->.nw;
is_in({n}, {e})->.ne;
is_in({s}, {e})->.se;
is_in({s}, {w})->.sw;
(
    way(pivot.nw)[landuse="winter_sports"];
    way(pivot.ne)[landuse="winter_sports"];
    way(pivot.se)[landuse="winter_sports"];
    way(pivot.sw)[landuse="winter_sports"];
    way({s}, {w}, {n}, {e})[landuse="winter_sports"];
);
out geom;"###,
        w = rect.min().x,
        s = rect.min().y,
        e = rect.max().x,
        n = rect.max().y,
    );
    query(query_string.as_str())
}
