use std::fs::{create_dir_all, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpx::Gpx;
use ski_analyzer_lib::error::{Error, ErrorType, Result};
use ski_analyzer_lib::gpx_analyzer::{analyze_route, AnalyzedRoute};
use ski_analyzer_lib::ski_area::SkiArea;
use tauri::{Manager, Runtime, Window, WindowEvent};

use super::config::Config;
use crate::utils::delayed_action::DelayedAction;

pub struct AppState {
    config_path: PathBuf,
    config: Option<Config>,
    window_saver: DelayedAction,
    ski_area: Option<SkiArea>,
    analyzed_route: Option<AnalyzedRoute>,
}

impl AppState {
    pub fn init_config<M: Manager<R>, R: Runtime>(
        &mut self,
        manager: &M,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.config_path = PathBuf::from(manager.path().data_dir()?);
        self.config_path.push("ski_analyzer");
        create_dir_all(&self.config_path)?;
        Ok(())
    }

    fn load_config(
        &mut self,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let res = OpenOptions::new()
            .read(true)
            .open(self.config_path.join("config.json"));
        match res {
            Ok(f) => {}
        };
        Ok(())
    }

    pub fn get_ski_area(&self) -> Option<&SkiArea> {
        self.ski_area.as_ref()
    }

    pub fn set_ski_area(&mut self, ski_area: SkiArea) {
        self.ski_area = Some(ski_area);
        self.analyzed_route = None;
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

    pub fn handle_window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) {
        let res = match event {
            WindowEvent::Moved(_) => self.save_window_config(window),
            WindowEvent::Resized(_) => self.save_window_config(window),
            _ => Ok(()),
        };
        if let Err(err) = res {
            eprintln!("Failed to save window position: {}", err);
            return;
        };
        //self.window_saver.call(Box::new(move || {}));
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            config_path: PathBuf::new(),
            config: None,
            window_saver: DelayedAction::new(Duration::from_secs(2)),
            ski_area: None,
            analyzed_route: None,
        }
    }
}
