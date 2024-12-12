use serde::{Deserialize, Serialize};
use tauri::{PhysicalPosition, PhysicalSize, Window};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    monitor_name: Option<String>,
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
    maximized: bool,
    fullscreen: bool,
}

fn get_relative_position(
    window: &Window,
) -> tauri::Result<PhysicalPosition<i32>> {
    let position = window.outer_position()?;
    Ok(match window.current_monitor()? {
        None => position,
        Some(monitor) => {
            let monitor_position = monitor.position();
            PhysicalPosition::new(
                position.x - monitor_position.x,
                position.y - monitor_position.y,
            )
        }
    })
}

fn get_monitor_name(window: &Window) -> tauri::Result<Option<String>> {
    Ok(window
        .current_monitor()?
        .and_then(|m| m.name().map(|n| n.clone())))
}

impl WindowConfig {
    pub fn new(window: &Window) -> tauri::Result<Self> {
        Ok(Self {
            monitor_name: get_monitor_name(window)?,
            position: get_relative_position(window)?,
            size: window.outer_size()?,
            maximized: window.is_maximized()?,
            fullscreen: window.is_fullscreen()?,
        })
    }

    pub fn update(&mut self, window: &Window) -> tauri::Result<()> {
        let new = Self::new(window)?;
        if !new.maximized && !new.fullscreen {
            *self = new;
        } else {
            self.maximized = new.maximized;
            self.fullscreen = new.fullscreen;
            self.monitor_name = new.monitor_name;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    window_config: Option<WindowConfig>,
}

impl Config {}
