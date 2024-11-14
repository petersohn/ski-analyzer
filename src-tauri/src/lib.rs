use app_state::AppState;
use ski_analyzer_lib::config::{set_config, Config};
use tauri::Manager;

use std::sync::Mutex;

mod app_state;
mod commands;

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
            commands::load_ski_area,
            commands::find_ski_area,
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
