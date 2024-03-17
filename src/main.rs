use clap::{Parser, Subcommand};
use config::{get_config, set_config, Config};
use osm_reader::Document;
use ski_area::SkiArea;

use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};

mod config;
mod error;
mod osm_query;
mod osm_reader;
mod ski_area;

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
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    set_config(args.config.clone())?;
    match args.command {
        Command::QueryOsm { name, output } => {
            let json = osm_query::query_ski_area(name.as_str())?;
            let mut file =
                OpenOptions::new().write(true).create(true).open(output)?;
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
                if get_config().verbose {
                    eprintln!(
                        "Total nodes: {}, total ways: {}",
                        doc.elements.nodes.len(),
                        doc.elements.ways.len(),
                    );
                }
                SkiArea::parse(&doc)?
            };

            let file =
                OpenOptions::new().write(true).create(true).open(output)?;
            if pretty {
                serde_json::to_writer_pretty(file, &ski_area)?;
            } else {
                serde_json::to_writer(file, &ski_area)?;
            }
        }
    };

    Ok(())
}
