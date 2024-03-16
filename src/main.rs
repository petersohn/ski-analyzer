use clap::Parser;
use osm_reader::Document;
use ski_area::SkiArea;

use std::error::Error;

mod error;
mod osm_reader;
mod ski_area;

#[derive(Parser)]
struct Args {
    /// Name of the ski area (case insensitive, regex)
    #[arg(short, long)]
    name: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let doc = Document::query(&args.name.as_str())?;
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
