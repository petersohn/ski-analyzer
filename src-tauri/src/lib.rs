use core::str;
use ski_analyzer_lib::config::{set_config, Config};
use ski_analyzer_lib::osm_query::query_ski_area;
use ski_analyzer_lib::osm_reader::Document;
use ski_analyzer_lib::ski_area::SkiArea;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Read;

fn load_file_inner(path: String) -> Result<String, Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(str::from_utf8(&data)?.to_string())
}

#[tauri::command]
fn load_file(path: String) -> Result<String, String> {
    load_file_inner(path).map_err(|e| e.to_string())
}

fn find_ski_area_inner(name: String) -> Result<SkiArea, Box<dyn Error>> {
    eprintln!("find ski area {}", name);
    let json = query_ski_area(name.as_str())?;
    let doc = Document::parse(&json)?;
    let ski_area = SkiArea::parse(&doc)?;
    Ok(ski_area)
}

#[tauri::command(async)]
fn find_ski_area(name: String) -> Result<SkiArea, String> {
    find_ski_area_inner(name).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    set_config(Config { verbose: 0 }).unwrap();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_file])
        .invoke_handler(tauri::generate_handler![find_ski_area])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
