use crate::app_state::AppStateType;
use crate::config::{CachedSkiArea, MapConfig};
use crate::task_manager::{do_with_task, TaskHandle, TaskManagerType};

use geo::{Intersects, Point, Rect};
use gpx::Waypoint;
use serde::{Deserialize, Deserializer, Serialize};
use ski_analyzer_lib::gpx_analyzer::{analyze_route, get_lines, DerivedData};
use ski_analyzer_lib::osm_query::{
    query_ski_area_details_by_id, query_ski_areas_by_coords,
    query_ski_areas_by_name,
};
use ski_analyzer_lib::osm_reader::Document;
use ski_analyzer_lib::ski_area::{SkiArea, SkiAreaMetadata};
use ski_analyzer_lib::utils::bounded_geometry::BoundedGeometry;
use ski_analyzer_lib::utils::gpx::load_from_file as load_gpx_from_file;
use ski_analyzer_lib::utils::json::{load_from_file, save_to_file};
use tauri::Manager;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use core::str;
use std::error::Error;

#[derive(Serialize)]
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
    let ski_area = load_from_file(&path)?;
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.set_ski_area(&app_handle, ski_area);
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
            Some((_, s)) => s.clone(),
        }
    };
    save_to_file(&ski_area, &path)?;
    Ok(())
}

#[tauri::command(async)]
pub fn save_current_ski_area_to_file(
    path: String,
    state: tauri::State<AppStateType>,
) -> Result<(), String> {
    save_current_ski_area_to_file_inner(path, state).map_err(|e| e.to_string())
}

async fn find_ski_areas_by_name_inner(
    task: TaskHandle,
    name: String,
) -> Result<Vec<SkiAreaMetadata>, Box<dyn Error>> {
    let json = task
        .add_async_task(
            async move { query_ski_areas_by_name(name.as_str()).await },
        )
        .await?;
    let doc = Document::parse(&json)?;
    Ok(SkiAreaMetadata::find(&doc)?)
}

#[tauri::command]
pub fn find_ski_areas_by_name(
    name: String,
    app_handle: tauri::AppHandle,
) -> u64 {
    do_with_task(app_handle, move |task| async move {
        find_ski_areas_by_name_inner(task, name)
            .await
            .map_err(|e| e.to_string())
    })
}

async fn find_ski_areas_by_coords_inner(
    task: TaskHandle,
    rect: Rect,
) -> Result<Vec<SkiAreaMetadata>, Box<dyn Error>> {
    let json = task.add_async_task(query_ski_areas_by_coords(rect)).await?;
    let doc = Document::parse(&json)?;
    Ok(SkiAreaMetadata::find(&doc)?)
}

#[tauri::command]
pub fn find_ski_areas_by_coords(
    rect: Rect,
    app_handle: tauri::AppHandle,
) -> u64 {
    do_with_task(app_handle, move |task| async move {
        find_ski_areas_by_coords_inner(task, rect)
            .await
            .map_err(|e| e.to_string())
    })
}

async fn load_ski_area_from_id_inner(
    task: TaskHandle,
    id: u64,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn Error>> {
    let json = task
        .add_async_task(query_ski_area_details_by_id(id))
        .await?;
    let doc = Document::parse(&json)?;
    let ski_area = task.add_sync_task(|cancel| SkiArea::parse(cancel, &doc))?;
    let state = app_handle.state::<AppStateType>();
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;

    app_state.set_ski_area(&app_handle, ski_area);
    Ok(())
}

#[tauri::command]
pub fn load_ski_area_from_id(id: u64, app_handle: tauri::AppHandle) -> u64 {
    do_with_task(app_handle.clone(), move |task| async move {
        load_ski_area_from_id_inner(task, id, app_handle)
            .await
            .map_err(|e| e.to_string())
    })
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
    task: TaskHandle,
    path: String,
    app_handle: tauri::AppHandle,
) -> Result<(), ski_analyzer_lib::error::Error> {
    let gpx = load_gpx_from_file(path)?;

    let state = app_handle.state::<AppStateType>();

    let (uuid, ski_area) = {
        let mut lock = state.inner().lock().unwrap();
        let line = BoundedGeometry::new(get_lines(&gpx))?;
        let cached = lock.get_current_cached_ski_area().ok_or_else(|| {
            ski_analyzer_lib::error::Error::new_s(
                ski_analyzer_lib::error::ErrorType::NoSkiAreaAtLocation(
                    line.bounding_rect,
                ),
                "No ski area loaded",
            )
        })?;
        if !cached.metadata.outline.intersects(&line) {
            return Err(ski_analyzer_lib::error::Error::new_s(
                ski_analyzer_lib::error::ErrorType::NoSkiAreaAtLocation(
                    line.bounding_rect,
                ),
                "Wrong ski area for GPX",
            ));
        }
        lock.get_clipped_ski_area().unwrap().clone()
    };

    let route =
        task.add_sync_task(|cancel| analyze_route(cancel, &ski_area, gpx))?;

    let mut lock = state.inner().lock().unwrap();
    if !lock.get_ski_area().map_or(false, |(u, _)| *u == uuid) {
        return Err(ski_analyzer_lib::error::Error::new_s(
            ski_analyzer_lib::error::ErrorType::Cancelled,
            "Ski area changed",
        ));
    }
    lock.set_route(&app_handle, route);

    Ok(())
}

#[tauri::command]
pub fn load_gpx(path: String, app_handle: tauri::AppHandle) -> u64 {
    do_with_task(app_handle.clone(), move |task| async move {
        load_gpx_inner(task, path, app_handle)
    })
}

fn load_route_inner(
    path: String,
    state: tauri::State<AppStateType>,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn Error>> {
    let route = load_from_file(&path)?;
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
    save_to_file(&route, &path)?;
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
    Ok(app_state.get_ski_area().map(|(_, s)| s.clone()))
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
    route
        .map(|r| serde_json::to_string(&r).map_err(|e| e.to_string()))
        .transpose()
}

#[derive(Deserialize, Debug)]
pub struct WaypointIn {
    point: Point,
    #[serde(default, deserialize_with = "parse_time")]
    time: Option<OffsetDateTime>,
    elevation: Option<f64>,
}

impl Into<Waypoint> for WaypointIn {
    fn into(self) -> Waypoint {
        let mut result = Waypoint::new(self.point);
        result.time = self.time.map(|t| t.into());
        result.elevation = self.elevation;
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
    time_str
        .map(|s| OffsetDateTime::parse(&s, &Rfc3339))
        .transpose()
        .map_err(serde::de::Error::custom)
}

#[tauri::command]
pub fn get_derived_data(wp1: WaypointIn, wp2: WaypointIn) -> DerivedData {
    DerivedData::calculate(&wp1.into(), &wp2.into())
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
            app_state.get_ski_area()?.1.get_closest_lift(p, limit)?;
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

#[tauri::command]
pub fn cancel_all_tasks(
    task_manager: tauri::State<TaskManagerType>,
) -> Result<(), String> {
    let mut lock = task_manager.inner().lock().map_err(|e| e.to_string())?;
    lock.cancel_all_tasks();
    Ok(())
}

#[tauri::command]
pub fn cancel_task(
    task_id: u64,
    task_manager: tauri::State<TaskManagerType>,
) -> Result<(), String> {
    let mut lock = task_manager.inner().lock().map_err(|e| e.to_string())?;
    lock.cancel_task(task_id);
    Ok(())
}

#[tauri::command]
pub fn get_ui_config(
    state: tauri::State<AppStateType>,
) -> Result<String, String> {
    let app_state = state.inner().lock().map_err(|e| e.to_string())?;
    Ok(app_state.get_ui_config())
}

#[tauri::command]
pub fn set_ui_config(
    state: tauri::State<AppStateType>,
    config: String,
) -> Result<(), String> {
    let mut app_state = state.inner().lock().map_err(|e| e.to_string())?;
    app_state.set_ui_config(config);
    Ok(())
}
