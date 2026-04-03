use clap::Parser;
use ski_analyzer_lib::gpx_analyzer::{
    Activity, ActivityType, DerivedData, MoveType, Moving, UseLift, WaypointDef,
};
use ski_analyzer_lib::ski_area::{
    Difficulty, Lift, Piste, PisteData, PisteMetadata, PointWithElevation,
    SkiArea, SkiAreaMetadata,
};
use ski_analyzer_lib::typescript_gen::geo::{
    LineStringDef, MultiLineStringDef, MultiPolygonDef, PointDef, PolygonDef,
    RectDef,
};
use ski_analyzer_lib::typescript_gen::gpx_analyzer::AnalyzedRouteDef;
use ski_analyzer_lib::typescript_gen::ski_analyzer::BoundedGeometryDef;
use specta_typescript::{BigIntExportBehavior, Typescript};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(about = "Generate TypeScript types from Rust types")]
struct Args {
    #[arg(short, long, default_value = "./generated")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let output_path = std::env::current_dir()?.join(&args.output);

    let types = specta::TypeCollection::default()
        .register::<PointDef>()
        .register::<RectDef>()
        .register::<LineStringDef>()
        .register::<MultiLineStringDef>()
        .register::<PolygonDef>()
        .register::<MultiPolygonDef>()
        .register::<BoundedGeometryDef<PolygonDef>>()
        .register::<BoundedGeometryDef<LineStringDef>>()
        .register::<PointWithElevation>()
        .register::<SkiAreaMetadata>()
        .register::<Lift>()
        .register::<Difficulty>()
        .register::<PisteMetadata>()
        .register::<PisteData>()
        .register::<Piste>()
        .register::<SkiArea>()
        .register::<WaypointDef>()
        .register::<UseLift>()
        .register::<MoveType>()
        .register::<Moving>()
        .register::<ActivityType>()
        .register::<Activity>()
        .register::<AnalyzedRouteDef>()
        .register::<DerivedData>();

    let output = Typescript::default()
        .bigint(BigIntExportBehavior::Number)
        .export(&types)?;

    fs::create_dir_all(&output_path)?;
    fs::write(output_path.join("generated.ts"), output)?;

    println!("TypeScript types written to: {}", output_path.display());

    Ok(())
}
