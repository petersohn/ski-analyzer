use app_state::AppState;
use ski_analyzer_lib::config::{set_config, Config};
use tauri::Manager;

use std::path::PathBuf;
use std::sync::Mutex;

mod app_state;
mod commands;
mod config;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    set_config(Config { verbose: 0 }).unwrap();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState::default()))
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
        .on_window_event(|window, event| {
            //eprintln!(
            //    "{:?}\n{} {} {:?} {:?} {:?}",
            //    event,
            //    window.is_maximized().unwrap_or(false),
            //    window.is_fullscreen().unwrap_or(false),
            //    window.current_monitor().unwrap(),
            //    window.outer_position().unwrap(),
            //    window.outer_size().unwrap(),
            //);
            window
                .state::<Mutex<AppState>>()
                .lock()
                .unwrap()
                .handle_window_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_ski_area_from_file,
            commands::find_ski_areas_by_name,
            commands::find_ski_areas_by_coords,
            commands::load_ski_area_from_id,
            commands::load_gpx,
            commands::load_route,
            commands::get_active_ski_area,
            commands::get_active_route,
            commands::get_speed,
            commands::get_closest_lift,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
