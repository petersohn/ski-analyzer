use std::error::Error;

use osm_reader::Document;
use ski_area::SkiArea;

mod error;
mod osm_reader;
mod ski_area;

fn main() -> Result<(), Box<dyn Error>> {
    let doc = Document::query("[out:json];rel(3545276);>;out;")?;
    eprintln!(
        "Total nodes: {}, total ways: {}",
        doc.elements.nodes.len(),
        doc.elements.ways.len(),
    );
    // println!("{:#?}", doc);
    // serde_json::to_writer(stdout(), &doc)?;

    let ski_area = SkiArea::parse(&doc);
    println!("{:#?}", ski_area);
    Ok(())
}
