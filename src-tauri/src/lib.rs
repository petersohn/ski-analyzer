use app_state::AppState;
use core::str;
use geo::{Distance, Haversine, Point};
use serde::Deserialize;
use ski_analyzer_lib::config::{set_config, Config};
use ski_analyzer_lib::osm_query::query_ski_area;
use ski_analyzer_lib::osm_reader::Document;
use ski_analyzer_lib::ski_area::SkiArea;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Read;
use std::sync::Mutex;
use tauri::Manager;

mod app_state;

fn load_ski_area_inner(
    path: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut json = Vec::new();
    file.read_to_end(&mut json)?;
    let ski_area = serde_json::from_slice(&json)?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.set_ski_area(ski_area);
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
    let doc = Document::parse(&json)?;
    let ski_area = SkiArea::parse(&doc)?;
    Ok(ski_area)
}

#[tauri::command(async)]
fn find_ski_area(
    name: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<SkiArea, String> {
    let ski_area = find_ski_area_inner(name).map_err(|e| e.to_string())?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.set_ski_area(ski_area.clone());
    Ok(ski_area)
}

fn load_gpx_inner(
    path: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, Box<dyn Error>> {
    let file = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(file);
    let gpx = gpx::read(reader)?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.set_gpx(gpx)?;
    Ok(serde_json::to_string(app_state.get_route().unwrap())?)
}

#[tauri::command(async)]
fn load_gpx(
    path: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, String> {
    load_gpx_inner(path, state).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_active_ski_area(
    state: tauri::State<Mutex<AppState>>,
) -> Result<Option<SkiArea>, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    Ok(app_state.get_ski_area().cloned())
}

#[tauri::command]
fn get_active_route(
    state: tauri::State<Mutex<AppState>>,
) -> Result<Option<String>, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    let route = app_state.get_route();
    let res = match route {
        None => None,
        Some(r) => Some(serde_json::to_string(&r).map_err(|e| e.to_string())?),
    };
    Ok(res)
}

#[derive(Deserialize)]
struct WaypointIn {
    point: Point,
    time: Option<f64>,
}

#[tauri::command]
fn get_speed(wp1: WaypointIn, wp2: WaypointIn) -> Option<f64> {
    match (wp1.time, wp2.time) {
        (Some(t1), Some(t2)) => {
            if t1 == t2 {
                None
            } else {
                Some(Haversine::distance(wp1.point, wp2.point) / (t2 - t1))
            }
        }
        _ => None,
    }
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
        .invoke_handler(tauri::generate_handler![
            load_ski_area,
            find_ski_area,
            load_gpx,
            get_active_ski_area,
            get_active_route,
            get_speed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
