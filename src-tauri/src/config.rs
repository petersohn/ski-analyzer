use std::collections::HashMap;

use geo::Point;
use serde::{Deserialize, Serialize};
use tauri::{PhysicalSize, Window};
use time::OffsetDateTime;
use uuid::Uuid;

use ski_analyzer_lib::ski_area::{SkiArea, SkiAreaMetadata};
use ski_analyzer_lib::utils::time_ser;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct WindowConfig {
    pub size: PhysicalSize<u32>,
    pub maximized: bool,
    pub fullscreen: bool,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct MapConfig {
    center: Point,
    zoom: f64,
}

fn update<T>(target: &mut T, source: T, changed: &mut bool)
where
    T: PartialEq,
{
    if *target != source {
        *target = source;
        *changed = true;
    }
}

impl WindowConfig {
    pub fn new(window: &Window) -> tauri::Result<Self> {
        Ok(Self {
            size: window.inner_size()?,
            maximized: window.is_maximized()?,
            fullscreen: window.is_fullscreen()?,
        })
    }

    pub fn update(&mut self, window: &Window) -> tauri::Result<bool> {
        let mut changed = false;
        let new = Self::new(window)?;
        if !new.maximized && !new.fullscreen {
            update(self, new, &mut changed);
        } else {
            update(&mut self.maximized, new.maximized, &mut changed);
            update(&mut self.fullscreen, new.fullscreen, &mut changed);
        }

        Ok(changed)
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
    pub fn save_window_config(
        &mut self,
        window: &Window,
    ) -> tauri::Result<bool> {
        let res = match self.window_config.as_mut() {
            Some(c) => c.update(window)?,
            None => {
                self.window_config = Some(WindowConfig::new(window)?);
                true
            }
        };
        Ok(res)
    }

    pub fn save_map_config(&mut self, map_config: MapConfig) -> bool {
        let mut changed = false;
        update(&mut self.map_config, Some(map_config), &mut changed);
        changed
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
