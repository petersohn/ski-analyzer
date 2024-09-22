use tauri::{Emitter, WindowEvent};

pub mod config;
pub mod error;
pub mod gpx_analyzer;
pub mod osm_query;
pub mod osm_reader;
pub mod ski_area;

mod collection;
mod multipolygon;

#[cfg(test)]
mod multipolygon_test;
#[cfg(test)]
mod osm_reader_test;
#[cfg(test)]
mod test_util;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        .on_page_load(|window, _payload| {
            window.emit("resized", &window.size().unwrap()).unwrap();
        })
        .on_window_event(|window, event| match event {
            WindowEvent::Resized(size) => {
                window.emit("resized", size).unwrap();
            }
            _ => (),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
