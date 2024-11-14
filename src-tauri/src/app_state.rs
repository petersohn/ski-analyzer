use gpx::Gpx;
use ski_analyzer_lib::error::{Error, ErrorType, Result};
use ski_analyzer_lib::gpx_analyzer::{analyze_route, AnalyzedRoute};
use ski_analyzer_lib::ski_area::SkiArea;

#[derive(Default)]
pub struct AppState {
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
}
