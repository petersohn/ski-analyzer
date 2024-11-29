use ski_analyzer_lib::config::{set_config, Config};
use ski_analyzer_lib::error::{convert_err, Error, ErrorType, Result};
use ski_analyzer_lib::gpx_analyzer::analyze_route;
use ski_analyzer_lib::osm_query::{
    query_ski_area_details_by_id, query_ski_areas_by_name,
};
use ski_analyzer_lib::osm_reader::Document;
use ski_analyzer_lib::ski_area::{SkiArea, SkiAreaMetadata};

use clap::{Args, Parser, Subcommand};
use gpx;
use serde::Serialize;

use std::fs::OpenOptions;
use std::io::BufReader;

#[derive(Parser)]
struct ArgParser {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    config: Config,
}

#[derive(Clone, Args)]
struct SerializedOutput {
    /// File name to save result
    #[arg(short, long)]
    output: String,
    /// Pretty print result
    #[arg(short, long)]
    pretty: bool,
}

impl SerializedOutput {
    fn write_to_file<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let file = convert_err(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&self.output),
            ErrorType::ExternalError,
        )?;
        convert_err(
            if self.pretty {
                serde_json::to_writer_pretty(file, &data)
            } else {
                serde_json::to_writer(file, &data)
            },
            ErrorType::ExternalError,
        )?;
        Ok(())
    }
}

#[derive(Clone, Subcommand)]
enum Command {
    /// Query ski area from OSM
    QueryOsm {
        /// Name of the ski area (case insensitive, regex)
        #[arg(short, long)]
        name: String,
        #[command(flatten)]
        output: SerializedOutput,
        /// Remove line parts from inside areas of the same piste.
        #[arg(short, long)]
        clip: bool,
    },
    Gpx {
        /// GPX file name
        #[arg(short, long)]
        input: String,
        /// Ski area to use (previously output from ParseOsm)
        #[arg(short, long)]
        area: String,
        #[command(flatten)]
        output: SerializedOutput,
    },
}

//fn query_osm

fn main() -> Result<()> {
    let args = ArgParser::parse();
    set_config(args.config.clone())?;
    match args.command {
        Command::QueryOsm { name, output, clip } => {
            let json1 = query_ski_areas_by_name(name.as_str())?;
            let doc1 = Document::parse(&json1)?;
            let metadatas = SkiAreaMetadata::find(&doc1);
            let id = match metadatas.len() {
                1 => metadatas.into_iter().next().unwrap().id,
                0 => {
                    return Err(Error::new_s(
                        ErrorType::InputError,
                        "ski area entity not found",
                    ));
                }
                _ => {
                    return Err(Error::new(
                        ErrorType::InputError,
                        format!("ambiguous ski area: {:?}", metadatas),
                    ));
                }
            };

            let json2 = query_ski_area_details_by_id(id)?;
            let doc2 = Document::parse(&json2)?;

            let mut ski_area = SkiArea::parse(&doc2)?;
            if clip {
                ski_area.clip_piste_lines();
            }

            convert_err(
                output.write_to_file(&ski_area),
                ErrorType::ExternalError,
            )?;
        }
        Command::Gpx {
            input,
            area,
            output,
        } => {
            let gpx: gpx::Gpx = {
                let file = convert_err(
                    OpenOptions::new().read(true).open(input),
                    ErrorType::ExternalError,
                )?;
                let reader = BufReader::new(file);
                convert_err(gpx::read(reader), ErrorType::ExternalError)?
            };

            // println!("{:#?}", gpx);

            let ski_area: SkiArea = {
                let file = convert_err(
                    OpenOptions::new().read(true).open(area),
                    ErrorType::ExternalError,
                )?;
                let reader = BufReader::new(file);
                convert_err(
                    serde_json::from_reader(reader),
                    ErrorType::ExternalError,
                )?
            };

            let result = analyze_route(&ski_area, gpx)?;
            output.write_to_file(&result)?;
        }
    };

    Ok(())
}
