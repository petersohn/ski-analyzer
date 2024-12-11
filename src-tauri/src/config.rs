use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WindowConfig {
    monitor_name: Option<String>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    maximized: bool,
    fullscreen: bool,
}

impl WindowConfig {
    fn new(window: tauri::Window) -> tauri::Result<Self> {
        let pos = window.outer_position()?;
        Ok(Self {
            monitor_name: window
                .current_monitor()?
                .and_then(|m| m.name().map(|n| n.clone())),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    //position
}
