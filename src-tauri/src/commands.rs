use crate::app_state::AppStateType;
use crate::config::{CachedSkiArea, MapConfig};

use geo::{Intersects, Point, Rect};
use gpx::Waypoint;
use serde::{Deserialize, Deserializer, Serialize};
use ski_analyzer_lib::osm_query::{
    query_ski_area_details_by_id, query_ski_areas_by_coords,
    query_ski_areas_by_name,
};
use ski_analyzer_lib::osm_reader::Document;
use ski_analyzer_lib::ski_area::{SkiArea, SkiAreaMetadata};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use core::str;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct CachedSkiAreaWithUuid {
    uuid: Uuid,
    #[serde(flatten)]
    data: CachedSkiArea,
}

impl CachedSkiAreaWithUuid {
    fn new((uuid, data): (&Uuid, &CachedSkiArea)) -> Self {
        Self {
            uuid: uuid.clone(),
            data: data.clone(),
        }
    }

    fn new_list<'a, It>(it: It) -> Vec<CachedSkiAreaWithUuid>
    where
        It: Iterator<Item = (&'a Uuid, &'a CachedSkiArea)>,
    {
        it.map(Self::new).collect()
    }
}

fn load_ski_area_from_file_inner(
    path: String,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new().read(true).open(path)?;
    let ski_area = serde_json::from_reader(&file)?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.set_ski_area(&app_handle, ski_area);
    app_state.save_ski_area(&app_handle);
    Ok(())
}

#[tauri::command(async)]
pub fn load_ski_area_from_file(
    path: String,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    load_ski_area_from_file_inner(path, state, app_handle)
        .map_err(|e| e.to_string())
}

fn save_current_ski_area_to_file_inner(
    path: String,
    state: tauri::State<AppStateType>,
) -> Result<(), Box<dyn Error>> {
    let ski_area = {
        let app_state = state.inner().lock().map_err(|e| e.to_string())?;
        match app_state.get_ski_area() {
            None => {
                return Err(Box::new(ski_analyzer_lib::error::Error::new_s(
                    ski_analyzer_lib::error::ErrorType::InputError,
                    "No active ski area",
                )))
            }
            Some(s) => s.clone(),
        }
    };
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    serde_json::to_writer(file, &ski_area)?;
    Ok(())
}

#[tauri::command(async)]
pub fn save_current_ski_area_to_file(
    path: String,
    state: tauri::State<AppStateType>,
) -> Result<(), String> {
    save_current_ski_area_to_file_inner(path, state).map_err(|e| e.to_string())
}

fn find_ski_areas_by_name_inner(
    name: String,
) -> Result<Vec<SkiAreaMetadata>, Box<dyn Error>> {
    let json = query_ski_areas_by_name(name.as_str())?;
    let doc = Document::parse(&json)?;
    Ok(SkiAreaMetadata::find(&doc)?)
}

#[tauri::command(async)]
pub fn find_ski_areas_by_name(
    name: String,
) -> Result<Vec<SkiAreaMetadata>, String> {
    find_ski_areas_by_name_inner(name).map_err(|e| e.to_string())
}

fn find_ski_areas_by_coords_inner(
    rect: Rect,
) -> Result<Vec<SkiAreaMetadata>, Box<dyn Error>> {
    let json = query_ski_areas_by_coords(rect)?;
    let doc = Document::parse(&json)?;
    Ok(SkiAreaMetadata::find(&doc)?)
}

#[tauri::command(async)]
pub fn find_ski_areas_by_coords(
    rect: Rect,
) -> Result<Vec<SkiAreaMetadata>, String> {
    find_ski_areas_by_coords_inner(rect).map_err(|e| e.to_string())
}

fn load_ski_area_from_id_inner(
    id: u64,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn Error>> {
    let json = query_ski_area_details_by_id(id)?;
    let doc = Document::parse(&json)?;
    let ski_area = SkiArea::parse(&doc)?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;

    app_state.set_ski_area(&app_handle, ski_area);
    app_state.save_ski_area(&app_handle);
    Ok(())
}

#[tauri::command(async)]
pub fn load_ski_area_from_id(
    id: u64,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    load_ski_area_from_id_inner(id, state, app_handle)
        .map_err(|e| e.to_string())
}

pub fn load_cached_ski_area_inner(
    uuid: Uuid,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn Error>> {
    let mut lock = state.inner().lock().unwrap();
    lock.load_cached_ski_area(&app_handle, &uuid)?;
    Ok(())
}

#[tauri::command]
pub fn load_cached_ski_area(
    uuid: Uuid,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    load_cached_ski_area_inner(uuid, state, app_handle)
        .map_err(|e| e.to_string())
}

fn load_gpx_inner(
    path: String,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(file);
    let gpx = gpx::read(reader)?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.set_gpx(&app_handle, gpx)?;
    Ok(())
}

#[tauri::command(async)]
pub fn load_gpx(
    path: String,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    load_gpx_inner(path, state, app_handle).map_err(|e| e.to_string())
}

fn load_route_inner(
    path: String,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(file);
    let route = serde_json::from_reader(reader)?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.set_route(&app_handle, route);
    Ok(())
}

#[tauri::command(async)]
pub fn load_route(
    path: String,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    load_route_inner(path, state, app_handle).map_err(|e| e.to_string())
}

fn save_current_route_to_file_inner(
    path: String,
    state: tauri::State<AppStateType>,
) -> Result<(), Box<dyn Error>> {
    let route = {
        let app_state = state.inner().lock().map_err(|e| e.to_string())?;
        match app_state.get_route() {
            None => {
                return Err(Box::new(ski_analyzer_lib::error::Error::new_s(
                    ski_analyzer_lib::error::ErrorType::InputError,
                    "No active route",
                )))
            }
            Some(r) => (*r).clone(),
        }
    };
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    serde_json::to_writer(file, &route)?;
    Ok(())
}

#[tauri::command(async)]
pub fn save_current_route_to_file(
    path: String,
    state: tauri::State<AppStateType>,
) -> Result<(), String> {
    save_current_route_to_file_inner(path, state).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_active_ski_area(
    state: tauri::State<AppStateType>,
) -> Result<Option<SkiArea>, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    Ok(app_state.get_ski_area().cloned())
}

#[tauri::command]
pub fn has_active_ski_area(
    state: tauri::State<AppStateType>,
) -> Result<bool, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    Ok(app_state.get_ski_area().is_some())
}

#[tauri::command]
pub fn get_active_route(
    state: tauri::State<AppStateType>,
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

impl Into<Waypoint> for WaypointIn {
    fn into(self) -> Waypoint {
        let mut result = Waypoint::new(self.point);
        result.time = self.time.map(|t| t.into());
        result
    }
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
    ski_analyzer_lib::gpx_analyzer::get_speed(&wp1.into(), &wp2.into())
}

#[derive(Serialize)]
pub struct ClosestLift {
    lift_id: String,
    distance: f64,
}

#[tauri::command]
pub fn get_closest_lift(
    state: tauri::State<AppStateType>,
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

#[tauri::command]
pub fn save_map_config(
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
    config: MapConfig,
) -> Result<(), String> {
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.save_map_config(&app_handle, config);
    Ok(())
}

#[tauri::command]
pub fn get_map_config(
    state: tauri::State<AppStateType>,
) -> Result<Option<MapConfig>, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    Ok(app_state.get_config().map_config)
}

#[tauri::command(async)]
pub fn get_all_cached_ski_areas(
    state: tauri::State<AppStateType>,
) -> Result<Vec<CachedSkiAreaWithUuid>, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    Ok(CachedSkiAreaWithUuid::new_list(
        app_state.get_cached_ski_areas().iter(),
    ))
}

#[tauri::command(async)]
pub fn get_cached_ski_areas_for_area(
    state: tauri::State<AppStateType>,
    rect: Rect,
) -> Result<Vec<CachedSkiAreaWithUuid>, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    let ski_areas = app_state.get_cached_ski_areas();

    Ok(CachedSkiAreaWithUuid::new_list(ski_areas.iter().filter(
        |(_, s)| s.metadata.outline.bounding_rect.intersects(&rect),
    )))
}

#[tauri::command(async)]
pub fn get_cached_ski_areas_by_name(
    state: tauri::State<AppStateType>,
    name: String,
) -> Result<Vec<CachedSkiAreaWithUuid>, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    let ski_areas = app_state.get_cached_ski_areas();
    let search_string = name.to_lowercase();

    Ok(CachedSkiAreaWithUuid::new_list(ski_areas.iter().filter(
        |(_, s)| s.metadata.name.to_lowercase().contains(&search_string),
    )))
}

#[tauri::command(async)]
pub fn remove_cached_ski_area(
    uuid: Uuid,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.remove_cached_ski_area(&app_handle, &uuid);
    Ok(())
}
