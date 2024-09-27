use core::str;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Read;
use tauri::{Emitter, WindowEvent};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
