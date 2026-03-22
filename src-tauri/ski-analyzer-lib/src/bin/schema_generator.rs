use schemars::schema_for;
use serde_json::Value;
use ski_analyzer_lib::json_schema::geo::{
    LineStringDef, MultiLineStringDef, MultiPolygonDef, PointDef, PolygonDef,
    RectDef,
};
use ski_analyzer_lib::json_schema::ski_analyzer::BoundedGeometryDef;
use ski_analyzer_lib::json_schema::time::OffsetDateTimeDef;
use ski_analyzer_lib::ski_area::{
    Difficulty, Lift, Piste, PisteData, PisteMetadata, PointWithElevation,
    SkiArea, SkiAreaMetadata,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR")
        .or_else(|_| env::var("SCHEMA_OUTPUT_DIR"))
        .unwrap_or_else(|_| "schemas".to_string());
    let schemas_dir = Path::new(&out_dir).join("schemas");
    fs::create_dir_all(&schemas_dir).unwrap();

    // Generate all schemas
    generate_schema::<PointDef>("point", &schemas_dir);
    generate_schema::<RectDef>("rect", &schemas_dir);
    generate_schema::<LineStringDef>("line-string", &schemas_dir);
    generate_schema::<MultiLineStringDef>("multi-line-string", &schemas_dir);
    generate_schema::<PolygonDef>("polygon", &schemas_dir);
    generate_schema::<MultiPolygonDef>("multi-polygon", &schemas_dir);
    generate_schema::<OffsetDateTimeDef>("offset-date-time", &schemas_dir);
    generate_schema::<PointWithElevation>("point-with-elevation", &schemas_dir);
    generate_schema::<BoundedGeometryDef<PolygonDef>>(
        "bounded-polygon",
        &schemas_dir,
    );
    generate_schema::<BoundedGeometryDef<LineStringDef>>(
        "bounded-line-string",
        &schemas_dir,
    );
    generate_schema::<Lift>("lift", &schemas_dir);
    generate_schema::<Difficulty>("difficulty", &schemas_dir);
    generate_schema::<PisteMetadata>("piste-metadata", &schemas_dir);
    generate_schema::<PisteData>("piste-data", &schemas_dir);
    generate_schema::<Piste>("piste", &schemas_dir);
    generate_schema::<SkiAreaMetadata>("ski-area-metadata", &schemas_dir);
    generate_schema::<SkiArea>("ski-area", &schemas_dir);

    // Post-process schemas to use external $ref
    post_process_schemas(&schemas_dir);

    println!("Schema directory: {}", schemas_dir.display());
    println!("cargo:schema_dir={}", schemas_dir.display());
}

fn generate_schema<T: schemars::JsonSchema>(name: &str, dir: &Path) {
    let schema = schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    fs::write(dir.join(format!("{}.json", name)), json).unwrap();
}

/// Maps old type names to (new_name, file_name)
fn get_type_renames() -> HashMap<&'static str, (&'static str, &'static str)> {
    let mut map = HashMap::new();
    // Geo types - rename from Def suffix
    map.insert("PointDef", ("Point", "point"));
    map.insert("RectDef", ("Rect", "rect"));
    map.insert("LineStringDef", ("LineString", "line-string"));
    map.insert(
        "MultiLineStringDef",
        ("MultiLineString", "multi-line-string"),
    );
    map.insert("PolygonDef", ("Polygon", "polygon"));
    map.insert("MultiPolygonDef", ("MultiPolygon", "multi-polygon"));
    // Time types
    map.insert("OffsetDateTimeDef", ("OffsetDateTime", "offset-date-time"));
    // Bounded geometry types - rename from verbose names
    map.insert(
        "PointWithElevation",
        ("PointWithElevation", "point-with-elevation"),
    );
    map.insert(
        "BoundedGeometryDef_for_PolygonDef",
        ("BoundedPolygon", "bounded-polygon"),
    );
    map.insert(
        "BoundedGeometryDef_for_LineStringDef",
        ("BoundedLineString", "bounded-line-string"),
    );
    // Ski area types - keep names
    map.insert("Lift", ("Lift", "lift"));
    map.insert("Difficulty", ("Difficulty", "difficulty"));
    map.insert("PisteMetadata", ("PisteMetadata", "piste-metadata"));
    map.insert("PisteData", ("PisteData", "piste-data"));
    map.insert("Piste", ("Piste", "piste"));
    map.insert("SkiAreaMetadata", ("SkiAreaMetadata", "ski-area-metadata"));
    map.insert("SkiArea", ("SkiArea", "ski-area"));
    map
}

/// Post-process all schemas to use external $ref and clean titles
fn post_process_schemas(schemas_dir: &Path) {
    let type_renames = get_type_renames();

    // Build mapping from old name to file name for ref replacement
    let type_to_file: HashMap<&str, &str> = type_renames
        .iter()
        .map(|(old, (_, file))| (*old, *file))
        .collect();

    // Collect all schema files
    let schema_files: Vec<String> = fs::read_dir(schemas_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                path.file_stem().map(|s| s.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    // Process each schema file
    for schema_name in &schema_files {
        let schema_path = schemas_dir.join(format!("{}.json", schema_name));
        let content = fs::read_to_string(&schema_path).unwrap();
        let mut schema: Value = serde_json::from_str(&content).unwrap();

        // Process the schema to use external refs
        process_schema(&mut schema, schema_name, &type_to_file);

        // Rename the title if needed
        rename_schema_titles(&mut schema, &type_renames);

        // Write back
        let json = serde_json::to_string_pretty(&schema).unwrap();
        fs::write(&schema_path, json).unwrap();
    }
}

/// Process a single schema to use external $ref where possible
fn process_schema(
    schema: &mut Value,
    current_file: &str,
    type_to_file: &HashMap<&str, &str>,
) {
    // First, replace internal refs with external refs throughout the schema
    replace_refs_with_external(schema, type_to_file);

    // Then, remove definitions that belong to other files
    let defs_to_remove: Vec<String> = {
        let defs = match schema.get("$defs") {
            Some(defs) => defs.as_object().unwrap(),
            None => return, // No definitions to process
        };

        type_to_file
            .iter()
            .filter(|(type_name, home_file)| {
                **home_file != current_file && defs.contains_key(**type_name)
            })
            .map(|(type_name, _)| type_name.to_string())
            .collect()
    };

    // Remove the definitions
    if let Some(defs) = schema.get_mut("$defs").and_then(|d| d.as_object_mut())
    {
        for type_name in &defs_to_remove {
            defs.remove(type_name);
        }

        // Clean up empty $defs
        if defs.is_empty() {
            schema.as_object_mut().unwrap().remove("$defs");
        }
    }
}

/// Recursively replace $ref from internal to external
fn replace_refs_with_external(
    value: &mut Value,
    type_to_file: &HashMap<&str, &str>,
) {
    match value {
        Value::Object(map) => {
            // Check if this is a $ref
            if let Some(ref_value) = map.get("$ref") {
                if let Some(ref_str) = ref_value.as_str() {
                    // Check if it's an internal ref like "#/$defs/TypeName"
                    if let Some(type_name) = ref_str.strip_prefix("#/$defs/") {
                        if let Some(home_file) = type_to_file.get(type_name) {
                            // Replace with external ref
                            map.insert(
                                "$ref".to_string(),
                                Value::String(format!("{}.json", home_file)),
                            );
                            return;
                        }
                    }
                }
            }

            // Recursively process all values in the object
            for (_, v) in map.iter_mut() {
                replace_refs_with_external(v, type_to_file);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                replace_refs_with_external(item, type_to_file);
            }
        }
        _ => {}
    }
}

/// Rename schema titles from old names to new clean names
fn rename_schema_titles(
    schema: &mut Value,
    type_renames: &HashMap<&str, (&str, &str)>,
) {
    // Build mapping from old title to new title
    let title_renames: HashMap<&str, &str> = type_renames
        .iter()
        .map(|(old, (new, _))| (*old, *new))
        .collect();

    // Rename the main title
    if let Some(title) = schema.get("title").and_then(|t| t.as_str()) {
        if let Some(new_title) = title_renames.get(title) {
            schema.as_object_mut().unwrap().insert(
                "title".to_string(),
                Value::String(new_title.to_string()),
            );
        }
    }

    // Rename titles in $defs
    if let Some(defs) = schema.get_mut("$defs").and_then(|d| d.as_object_mut())
    {
        let defs_to_rename: Vec<(String, String)> = defs
            .keys()
            .filter_map(|name| {
                if let Some((new_name, _)) = type_renames.get(name.as_str()) {
                    if *new_name != name.as_str() {
                        return Some((name.clone(), new_name.to_string()));
                    }
                }
                None
            })
            .collect();

        for (old_name, new_name) in defs_to_rename {
            if let Some(mut def) = defs.remove(&old_name) {
                // Update the title inside the definition
                if let Some(obj) = def.as_object_mut() {
                    obj.insert(
                        "title".to_string(),
                        Value::String(new_name.clone()),
                    );
                }
                defs.insert(new_name, def);
            }
        }
    }
}
