use ski_analyzer_lib::config::{get_config, set_config, Config};
use ski_analyzer_lib::gpx_analyzer::analyze_route;
use ski_analyzer_lib::osm_query::query_ski_area;
use ski_analyzer_lib::osm_reader::Document;
use ski_analyzer_lib::ski_area::SkiArea;

use clap::{Parser, Subcommand};
use gpx;

use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufReader, Read, Write};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    config: Config,
}

#[derive(Clone, Subcommand)]
enum Command {
    /// Query ski area from OSM
    QueryOsm {
        /// Name of the ski area (case insensitive, regex)
        #[arg(short, long)]
        name: String,
        /// File name to save result
        #[arg(short, long)]
        output: String,
    },
    /// Parse ski area from JSON file
    ParseOsm {
        /// File name (previously output from QueryOsm)
        #[arg(short, long)]
        input: String,
        /// File name to save result
        #[arg(short, long)]
        output: String,
        /// Pretty print result
        #[arg(short, long)]
        pretty: bool,
    },
    Gpx {
        /// GPX file name
        #[arg(short, long)]
        input: String,
        /// Ski area to use (previously output from ParseOsm)
        #[arg(short, long)]
        area: String,
        /// File name to save result
        #[arg(short, long)]
        output: String,
        /// Pretty print result
        #[arg(short, long)]
        pretty: bool,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    set_config(args.config.clone())?;
    match args.command {
        Command::QueryOsm { name, output } => {
            let json = query_ski_area(name.as_str())?;
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)?;
            file.write(&json)?;
        }
        Command::ParseOsm {
            input,
            output,
            pretty,
        } => {
            let ski_area = {
                let mut file = OpenOptions::new().read(true).open(input)?;
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                let doc = Document::parse(&data)?;
                if get_config().is_v() {
                    eprintln!(
                        "Total nodes: {}, total ways: {}",
                        doc.elements.nodes.len(),
                        doc.elements.ways.len(),
                    );
                }
                SkiArea::parse(&doc)?
            };

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)?;
            if pretty {
                serde_json::to_writer_pretty(file, &ski_area)?;
            } else {
                serde_json::to_writer(file, &ski_area)?;
            }
        }
        Command::Gpx {
            input,
            area,
            output,
            pretty,
        } => {
            let gpx: gpx::Gpx = {
                let file = OpenOptions::new().read(true).open(input)?;
                let reader = BufReader::new(file);
                gpx::read(reader)?
            };

            // println!("{:#?}", gpx);

            let ski_area: SkiArea = {
                let file = OpenOptions::new().read(true).open(area)?;
                let reader = BufReader::new(file);
                serde_json::from_reader(reader)?
            };

            let result = analyze_route(&ski_area, &gpx);

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)?;
            if pretty {
                serde_json::to_writer_pretty(file, &result)?;
            } else {
                serde_json::to_writer(file, &result)?;
            }
        }
    };

    Ok(())
}
