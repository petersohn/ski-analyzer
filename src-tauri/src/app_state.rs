use std::time::Duration;

use crate::utils::delayed_action::DelayedAction;
use gpx::Gpx;
use ski_analyzer_lib::error::{Error, ErrorType, Result};
use ski_analyzer_lib::gpx_analyzer::{analyze_route, AnalyzedRoute};
use ski_analyzer_lib::ski_area::SkiArea;
use tauri::{Window, WindowEvent};

pub struct AppState {
    window_saver: DelayedAction,
    ski_area: Option<SkiArea>,
    analyzed_route: Option<AnalyzedRoute>,
}

impl AppState {
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

    fn save_window_config(&mut self, window: &Window) -> tauri::Result<()> {
        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            window_saver: DelayedAction::new(Duration::from_secs(2)),
            ski_area: None,
            analyzed_route: None,
        }
    }
}
