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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
