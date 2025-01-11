use crate::config::get_config;
use crate::error::{Error, ErrorType, Result};
use geo::Rect;
use url::form_urlencoded;

async fn query_inner(query: &str) -> reqwest::Result<Vec<u8>> {
    let mut input: String =
        form_urlencoded::byte_serialize(&query.as_bytes()).collect();
    input.insert_str(0, "data=");

    if get_config().is_vv() {
        eprintln!("{}", input);
    }

    let client = reqwest::Client::new();
    Ok(client
        .post("https://overpass-api.de/api/interpreter")
        .body(input)
        .send()
        .await?
        .bytes()
        .await?
        .into())
}

pub async fn query(query: &str) -> Result<Vec<u8>> {
    Ok(query_inner(&query).await.or_else(|err| {
        Err(Error::convert(
            ErrorType::ExternalError,
            "query error",
            &err,
        ))
    })?)
}

pub async fn query_ski_area_details_by_id(id: u64) -> Result<Vec<u8>> {
    let query_string = format!(
        r###"[out:json];
(
    (
        way({})->.a;
        way(area.a)["aerialway"];
        way(area.a)["piste:type"="downhill"];
        rel(area.a)["piste:type"="downhill"];
    );
    >;
);
out;"###,
        id
    );
    query(query_string.as_str()).await
}

pub async fn query_ski_areas_by_name(name: &str) -> Result<Vec<u8>> {
    let query_string = format!(
        r###"[out:json];
way[landuse="winter_sports"][name~"{}",i];
out geom;"###,
        name
    );
    query(query_string.as_str()).await
}

pub async fn query_ski_areas_by_coords(rect: Rect) -> Result<Vec<u8>> {
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
    query(query_string.as_str()).await
}
