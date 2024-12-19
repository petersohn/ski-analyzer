use geo::Point;
use serde::{Deserialize, Serialize};
use tauri::{PhysicalSize, Window};
use time::OffsetDateTime;

use ski_analyzer_lib::ski_area::SkiAreaMetadata;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct WindowConfig {
    pub size: PhysicalSize<u32>,
    pub maximized: bool,
    pub fullscreen: bool,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct MapConfig {
    center: Point,
    zoom: f64,
}

impl WindowConfig {
    pub fn new(window: &Window) -> tauri::Result<Self> {
        Ok(Self {
            size: window.inner_size()?,
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
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedSkiArea {
    pub file_name: String,
    pub date: OffsetDateTime,
    #[serde(flatten)]
    pub metadata: SkiAreaMetadata,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub window_config: Option<WindowConfig>,
    pub map_config: Option<MapConfig>,
}

impl Config {
    pub fn save_window_config(&mut self, window: &Window) -> tauri::Result<()> {
        match self.window_config.as_mut() {
            Some(c) => c.update(window)?,
            None => self.window_config = Some(WindowConfig::new(window)?),
        }
        Ok(())
    }
}
