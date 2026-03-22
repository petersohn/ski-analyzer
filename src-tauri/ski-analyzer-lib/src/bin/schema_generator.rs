use schemars::schema_for;
use ski_analyzer_lib::json_schema::geo::{
    LineStringDef, MultiLineStringDef, MultiPolygonDef, PointDef, PolygonDef,
    RectDef,
};
use ski_analyzer_lib::json_schema::ski_analyzer::BoundedGeometryDef;
use ski_analyzer_lib::json_schema::time::OffsetDateTimeDef;
use ski_analyzer_lib::ski_area::{PointWithElevation, SkiArea};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR")
        .or_else(|_| env::var("SCHEMA_OUTPUT_DIR"))
        .unwrap_or_else(|_| "schemas".to_string());
    let schemas_dir = Path::new(&out_dir).join("schemas");
    fs::create_dir_all(&schemas_dir).unwrap();

    // Geo types (used in TypeScript for map rendering)
    generate_schema::<PointDef>("point", &schemas_dir);
    generate_schema::<RectDef>("rect", &schemas_dir);
    generate_schema::<LineStringDef>("line-string", &schemas_dir);
    generate_schema::<MultiLineStringDef>("multi-line-string", &schemas_dir);
    generate_schema::<PolygonDef>("polygon", &schemas_dir);
    generate_schema::<MultiPolygonDef>("multi-polygon", &schemas_dir);

    // Time types
    generate_schema::<OffsetDateTimeDef>("offset-date-time", &schemas_dir);

    // Bounded geometry types (used in TypeScript for map rendering)
    generate_schema::<PointWithElevation>("point-with-elevation", &schemas_dir);
    generate_schema::<BoundedGeometryDef<PolygonDef>>(
        "bounded-polygon",
        &schemas_dir,
    );
    generate_schema::<BoundedGeometryDef<LineStringDef>>(
        "bounded-line-string",
        &schemas_dir,
    );

    // SkiArea is the primary type sent from Rust to TypeScript
    // It includes all sub-types (Lift, Piste, etc.) inline
    generate_schema::<SkiArea>("ski-area", &schemas_dir);

    println!("Schema directory: {}", schemas_dir.display());
    println!("cargo:schema_dir={}", schemas_dir.display());
}

fn generate_schema<T: schemars::JsonSchema>(name: &str, dir: &Path) {
    let schema = schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    fs::write(dir.join(format!("{}.json", name)), json).unwrap();
}
