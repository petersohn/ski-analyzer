use ski_analyzer_lib::config::{set_config, Config};
use tokio::runtime::Builder;

use ski_analyzer_lib::error::Result;
use ski_analyzer_lib::osm_query::query;
use ski_analyzer_lib::osm_reader::Document;

async fn run(id: u64) -> Result<()> {
    let query_string = format!("[out:json];way({id});out geom;");
    let query_result = query(&query_string).await?;
    let document = Document::parse(&query_result)?;
    let way = document.elements.get_way(&id)?;
    for g in &way.geometry {
        println!("[{}, {}],", g.lon, g.lat);
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let id: u64 = args[1].parse().unwrap();
    set_config(Config { verbose: 0 }).unwrap();

    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(run(id)).unwrap();
}
