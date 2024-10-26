use app_state::AppState;
use core::str;
use ski_analyzer_lib::config::{set_config, Config};
use ski_analyzer_lib::osm_query::query_ski_area;
use ski_analyzer_lib::osm_reader::Document;
use ski_analyzer_lib::ski_area::SkiArea;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Read;
use std::sync::Mutex;
use tauri::Manager;

mod app_state;

fn load_file_inner(path: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn parse_ski_area(json: &[u8]) -> ski_analyzer_lib::error::Result<SkiArea> {
    let doc = Document::parse(&json)?;
    let ski_area = SkiArea::parse(&doc)?;
    Ok(ski_area)
}

fn load_ski_area_inner(
    path: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, Box<dyn Error>> {
    let json = load_file_inner(path)?;
    let ski_area = parse_ski_area(&json)?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.active_ski_area = Some(ski_area);
    Ok(str::from_utf8(&json)?.to_string())
}

#[tauri::command]
fn load_ski_area(
    path: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, String> {
    load_ski_area_inner(path, state).map_err(|e| e.to_string())
}

fn find_ski_area_inner(name: String) -> Result<SkiArea, Box<dyn Error>> {
    eprintln!("find ski area {}", name);
    let json = query_ski_area(name.as_str())?;
    let ski_area = parse_ski_area(&json)?;
    Ok(ski_area)
}

#[tauri::command(async)]
fn find_ski_area(
    name: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<SkiArea, String> {
    let ski_area = find_ski_area_inner(name).map_err(|e| e.to_string())?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.active_ski_area = Some(ski_area.clone());
    Ok(ski_area)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    set_config(Config { verbose: 0 }).unwrap();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_ski_area, find_ski_area])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
