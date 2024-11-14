use crate::app_state::AppState;
use geo::{Distance, Haversine, Point};
use serde::{Deserialize, Deserializer, Serialize};
use ski_analyzer_lib::gpx_analyzer::AnalyzedRoute;
use ski_analyzer_lib::osm_query::query_ski_area;
use ski_analyzer_lib::osm_reader::Document;
use ski_analyzer_lib::ski_area::SkiArea;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use core::str;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Read;
use std::sync::Mutex;

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
pub fn load_ski_area(
    path: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, String> {
    load_ski_area_inner(path, state).map_err(|e| e.to_string())
}

fn find_ski_area_inner(name: String) -> Result<SkiArea, Box<dyn Error>> {
    let json = query_ski_area(name.as_str())?;
    let doc = Document::parse(&json)?;
    let ski_area = SkiArea::parse(&doc)?;
    Ok(ski_area)
}

#[tauri::command(async)]
pub fn find_ski_area(
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
pub fn load_gpx(
    path: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, String> {
    load_gpx_inner(path, state).map_err(|e| e.to_string())
}

fn load_route_inner(
    path: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, Box<dyn Error>> {
    let file = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(file);
    let route = serde_json::from_reader(reader)?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.set_route(route);
    Ok(serde_json::to_string(app_state.get_route().unwrap())?)
}

#[tauri::command(async)]
pub fn load_route(
    path: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, String> {
    load_route_inner(path, state).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_active_ski_area(
    state: tauri::State<Mutex<AppState>>,
) -> Result<Option<SkiArea>, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    Ok(app_state.get_ski_area().cloned())
}

#[tauri::command]
pub fn get_active_route(
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

#[derive(Deserialize, Debug)]
pub struct WaypointIn {
    point: Point,
    #[serde(default, deserialize_with = "parse_time")]
    time: Option<OffsetDateTime>,
}

fn parse_time<'de, D>(
    deserializer: D,
) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let time_str: Option<String> = Option::deserialize(deserializer)?;
    match time_str {
        None => Ok(None),
        Some(s) => {
            let t = OffsetDateTime::parse(&s, &Rfc3339)
                .map_err(serde::de::Error::custom)?;
            Ok(Some(t))
        }
    }
}

#[tauri::command]
pub fn get_speed(wp1: WaypointIn, wp2: WaypointIn) -> Option<f64> {
    let t1 = wp1.time?;
    let t2 = wp2.time?;
    if t1 == t2 {
        None
    } else {
        let t = (t2 - t1).as_seconds_f64();
        Some(Haversine::distance(wp1.point, wp2.point) / t)
    }
}

#[derive(Serialize)]
pub struct ClosestLift {
    lift_id: String,
    distance: f64,
}

#[tauri::command]
pub fn get_closest_lift(
    state: tauri::State<Mutex<AppState>>,
    p: Point,
    limit: f64,
) -> Result<Option<ClosestLift>, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    Ok((|| {
        let (lift_id, distance) =
            app_state.get_ski_area()?.get_closest_lift(p, limit)?;
        Some(ClosestLift {
            lift_id: lift_id.to_string(),
            distance,
        })
    })())
}
