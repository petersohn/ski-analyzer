use ski_analyzer_lib::config::{set_config, Config};
use ski_analyzer_lib::error::{Error, ErrorType, Result};
use ski_analyzer_lib::gpx_analyzer::analyze_route;
use ski_analyzer_lib::osm_query::{
    query_ski_area_details_by_id, query_ski_areas_by_name,
};
use ski_analyzer_lib::osm_reader::Document;
use ski_analyzer_lib::ski_area::{SkiArea, SkiAreaMetadata};
use ski_analyzer_lib::utils::cancel::CancellationToken;
use ski_analyzer_lib::utils::gpx::load_from_file as load_gpx;
use ski_analyzer_lib::utils::json::{
    load_from_file, save_to_file, save_to_file_pretty,
};

use clap::{Args, Parser, Subcommand};
use serde::Serialize;

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
        if self.pretty {
            save_to_file_pretty(data, &self.output)
        } else {
            save_to_file(data, &self.output)
        }
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = ArgParser::parse();
    set_config(args.config.clone())?;
    match args.command {
        Command::QueryOsm { name, output, clip } => {
            let json1 = query_ski_areas_by_name(name.as_str()).await?;
            let doc1 = Document::parse(&json1)?;
            let metadatas = SkiAreaMetadata::find(&doc1)?;
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

            let json2 = query_ski_area_details_by_id(id).await?;
            let doc2 = Document::parse(&json2)?;

            let mut ski_area =
                SkiArea::parse(&CancellationToken::new(), &doc2)?;
            if clip {
                ski_area.clip_piste_lines();
            }

            output.write_to_file(&ski_area)?;
        }
        Command::Gpx {
            input,
            area,
            output,
        } => {
            let gpx = load_gpx(input)?;

            // println!("{:#?}", gpx);

            let ski_area: SkiArea = load_from_file(area)?;

            let result =
                analyze_route(&CancellationToken::new(), &ski_area, gpx)?;
            output.write_to_file(&result)?;
        }
    };

    Ok(())
}
