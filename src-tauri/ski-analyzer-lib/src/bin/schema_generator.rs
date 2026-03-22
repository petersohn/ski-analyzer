use schemars::SchemaGenerator;
use ski_analyzer_lib::json_schema::geo::{
    LineStringDef, MultiLineStringDef, MultiPolygonDef, PointDef, PolygonDef,
    RectDef,
};
use ski_analyzer_lib::json_schema::ski_analyzer::BoundedGeometryDef;
use ski_analyzer_lib::json_schema::time::OffsetDateTimeDef;
use ski_analyzer_lib::ski_area::{
    Difficulty, Lift, PisteData, PisteMetadata, PointWithElevation, SkiArea,
    SkiAreaMetadata,
};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR")
        .or_else(|_| env::var("SCHEMA_OUTPUT_DIR"))
        .unwrap_or_else(|_| "schemas".to_string());
    let schemas_dir = Path::new(&out_dir).join("schemas");
    // Remove dir and ignore error (it might not exist
    let _ = fs::remove_dir_all(&schemas_dir);
    fs::create_dir_all(&schemas_dir).unwrap();

    let mut generator = SchemaGenerator::default();

    // Geo types (used in TypeScript for map rendering)
    generator.subschema_for::<PointDef>();
    generator.subschema_for::<RectDef>();
    generator.subschema_for::<LineStringDef>();
    generator.subschema_for::<MultiLineStringDef>();
    generator.subschema_for::<PolygonDef>();
    generator.subschema_for::<MultiPolygonDef>();

    // Time types
    generator.subschema_for::<OffsetDateTimeDef>();

    generator.subschema_for::<PointWithElevation>();

    // Bounded geometry types (used in TypeScript for map rendering)
    generator.subschema_for::<BoundedGeometryDef<PolygonDef>>();
    generator.subschema_for::<BoundedGeometryDef<LineStringDef>>();

    generator.subschema_for::<Lift>();
    generator.subschema_for::<SkiAreaMetadata>();
    generator.subschema_for::<Difficulty>();
    generator.subschema_for::<PisteMetadata>();
    generator.subschema_for::<PisteData>();
    generator.subschema_for::<SkiArea>();

    let json = serde_json::to_string_pretty(generator.definitions()).unwrap();
    fs::write(schemas_dir.join("ski-analyzer.json"), json).unwrap();

    println!("Schema directory: {}", schemas_dir.display());
}
