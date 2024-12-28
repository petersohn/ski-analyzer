use std::collections::HashMap;
use std::fs::{create_dir_all, OpenOptions};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpx::Gpx;
use ski_analyzer_lib::error::{Error, ErrorType, Result};
use ski_analyzer_lib::gpx_analyzer::{analyze_route, AnalyzedRoute};
use ski_analyzer_lib::ski_area::SkiArea;
use tauri::{
    Manager, PhysicalPosition, Position, Runtime, Size, Window, WindowEvent,
};
use uuid::Uuid;

use crate::config::{CachedSkiArea, Config, MapConfig, WindowConfig};
use crate::utils::delayed_action::DelayedAction;

pub struct AppState {
    config_path: PathBuf,
    config_file_path: PathBuf,
    ski_areas_path: PathBuf,
    config: Option<Config>,
    window_initialized: bool,
    window_saver: DelayedAction,
    ski_area: Option<SkiArea>,
    analyzed_route: Option<AnalyzedRoute>,
}

impl AppState {
    pub fn init_config<M: Manager<R>, R: Runtime>(&mut self, manager: &M) {
        self.config_path = PathBuf::from(manager.path().data_dir().unwrap());
        self.config_path.push("ski-analyzer");
        self.config_file_path = self.config_path.join("config.json");
        self.ski_areas_path = self.config_path.join("ski_areas");
        let config = match self.load_config() {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Failed to load config: {}", err);
                Config::default()
            }
        };
        self.config = Some(config);
    }

    fn load_config(
        &self,
    ) -> std::result::Result<Config, Box<dyn std::error::Error>> {
        let file =
            OpenOptions::new().read(true).open(&self.config_file_path)?;
        Ok(serde_json::from_reader(file)?)
    }

    fn save_config_inner(
        &self,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        create_dir_all(&self.config_path)?;
        eprintln!("save config -> {:?}", self.config_file_path);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.config_file_path)?;
        serde_json::to_writer(file, self.get_config())?;

        Ok(())
    }

    fn save_config(&self) {
        if let Err(err) = self.save_config_inner() {
            eprintln!("Failed to save config: {}", err);
        }
    }

    fn save_config_delayed<M: Manager<R>, R: Runtime>(&mut self, manager: &M) {
        let state = Arc::clone(manager.state::<AppStateType>().inner());
        self.window_saver.call(Box::new(move || {
            state.lock().unwrap().save_config();
        }));
    }

    pub fn get_config(&self) -> &Config {
        self.config.as_ref().unwrap()
    }

    fn get_config_mut(&mut self) -> &mut Config {
        self.config.as_mut().unwrap()
    }

    pub fn get_ski_area(&self) -> Option<&SkiArea> {
        self.ski_area.as_ref()
    }

    pub fn set_ski_area(&mut self, ski_area: SkiArea) {
        let uuid = self.get_config_mut().save_ski_area(&ski_area);
        self.ski_area = Some(ski_area);
        self.analyzed_route = None;

        if let Err(err) = self.save_ski_area(&uuid) {
            eprintln!("Failed to save ski area: {}", err);
            self.get_config_mut().remove_ski_area(&uuid);
        }
    }

    fn save_ski_area(
        &mut self,
        uuid: &Uuid,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.window_saver.cancel();
        self.save_config_inner()?;
        create_dir_all(&self.ski_areas_path)?;
        let path = self.get_ski_area_path(uuid);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        serde_json::to_writer(file, self.ski_area.as_ref().unwrap())?;

        Ok(())
    }

    fn get_ski_area_path(&self, uuid: &Uuid) -> PathBuf {
        self.ski_areas_path.join(format!("{}.json", uuid))
    }

    pub fn load_cached_ski_area(
        &mut self,
        uuid: &Uuid,
    ) -> std::result::Result<SkiArea, Box<dyn std::error::Error>> {
        let file = OpenOptions::new()
            .read(true)
            .open(&self.get_ski_area_path(uuid));
        if let Err(err) = &file {
            if err.kind() == std::io::ErrorKind::NotFound {
                self.remove_cached_ski_area(uuid);
            }
        }
        Ok(serde_json::from_reader(file?)?)
    }

    pub fn get_cached_ski_areas(&self) -> &HashMap<Uuid, CachedSkiArea> {
        &self.get_config().ski_areas
    }

    pub fn remove_cached_ski_area(&mut self, uuid: &Uuid) {
        self.get_config_mut().remove_ski_area(uuid);
        self.save_config();
    }

    pub fn get_route(&self) -> Option<&AnalyzedRoute> {
        self.analyzed_route.as_ref()
    }

    pub fn set_route(&mut self, route: AnalyzedRoute) {
        self.analyzed_route = Some(route);
    }

    pub fn set_gpx(&mut self, gpx: Gpx) -> Result<()> {
        let ski_area = self.ski_area.as_ref().ok_or_else(|| {
            Error::new_s(ErrorType::LogicError, "No ski area loaded")
        })?;
        self.analyzed_route = Some(analyze_route(ski_area, gpx)?);
        Ok(())
    }

    pub fn save_map_config<M: Manager<R>, R: Runtime>(
        &mut self,
        manager: &M,
        config: MapConfig,
    ) {
        self.get_config_mut().map_config = Some(config);
        self.save_config_delayed(manager);
    }

    pub fn handle_window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) {
        let res = match event {
            WindowEvent::Resized(_) => self.save_window_config(window),
            _ => Ok(()),
        };
        if let Err(err) = res {
            eprintln!("Failed to save window position: {}", err);
            return;
        };
    }

    fn save_window_config(&mut self, window: &Window) -> tauri::Result<()> {
        if !self.window_initialized {
            self.window_initialized = true;
            if let Some(c) = &self.get_config().window_config {
                return self.init_window(window, c);
            }
        }

        self.get_config_mut().save_window_config(window)?;
        self.save_config_delayed(window);
        Ok(())
    }

    fn init_window(
        &self,
        window: &Window,
        config: &WindowConfig,
    ) -> tauri::Result<()> {
        window.set_size(Size::Physical(config.size))?;
        if let Some(monitor) = window.current_monitor()? {
            let ms = monitor.size();
            let mp = monitor.position();
            let ws = &config.size;
            let pos = PhysicalPosition::new(
                mp.x + (ms.width as i32 - ws.width as i32) / 2,
                mp.y + (ms.height as i32 - ws.height as i32) / 2,
            );
            window.set_position(Position::Physical(pos))?;
        }
        if config.maximized {
            window.maximize()?;
        }
        if config.fullscreen {
            window.set_fullscreen(true)?;
        }

        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            config_path: PathBuf::new(),
            config_file_path: PathBuf::new(),
            ski_areas_path: PathBuf::new(),
            config: None,
            window_initialized: false,
            window_saver: DelayedAction::new(Duration::from_secs(2)),
            ski_area: None,
            analyzed_route: None,
        }
    }
}

pub type AppStateType = Arc<Mutex<AppState>>;
