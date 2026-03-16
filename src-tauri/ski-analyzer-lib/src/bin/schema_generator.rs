use schemars::schema_for;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR")
        .or_else(|_| env::var("SCHEMA_OUTPUT_DIR"))
        .unwrap_or_else(|_| "schemas".to_string());
    let schemas_dir = Path::new(&out_dir).join("schemas");
    fs::create_dir_all(&schemas_dir).unwrap();

    println!("Schema directory: {}", schemas_dir.display());
    println!("cargo:schema_dir={}", schemas_dir.display());
}

#[allow(dead_code)]
fn generate_schema<T: schemars::JsonSchema>(name: &str, dir: &Path) {
    let schema = schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    fs::write(dir.join(format!("{}.json", name)), json).unwrap();
}
