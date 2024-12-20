use std::collections::HashMap;

use geo::Point;
use serde::{Deserialize, Serialize};
use tauri::{PhysicalSize, Window};
use time::OffsetDateTime;
use uuid::Uuid;

use ski_analyzer_lib::ski_area::{SkiArea, SkiAreaMetadata};
use ski_analyzer_lib::utils::time_ser;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedSkiArea {
    pub metadata: SkiAreaMetadata,
    #[serde(with = "time_ser")]
    pub date: OffsetDateTime,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub window_config: Option<WindowConfig>,
    pub map_config: Option<MapConfig>,
    #[serde(default)]
    pub ski_areas: HashMap<Uuid, CachedSkiArea>,
}

impl Config {
    pub fn save_window_config(&mut self, window: &Window) -> tauri::Result<()> {
        match self.window_config.as_mut() {
            Some(c) => c.update(window)?,
            None => self.window_config = Some(WindowConfig::new(window)?),
        }
        Ok(())
    }

    pub fn save_ski_area(&mut self, ski_area: &SkiArea) -> Uuid {
        let uuid = Uuid::new_v4();
        self.ski_areas.insert(
            uuid,
            CachedSkiArea {
                metadata: ski_area.metadata.clone(),
                date: ski_area.date,
            },
        );
        uuid
    }

    pub fn remove_ski_area(&mut self, uuid: &Uuid) {
        self.ski_areas.remove(uuid);
    }
}
