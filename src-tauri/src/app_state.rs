use gpx::Gpx;
use ouroboros::self_referencing;
use ski_analyzer_lib::error::{Error, ErrorType, Result};
use ski_analyzer_lib::gpx_analyzer::{analyze_route, AnalyzedRoute};
use ski_analyzer_lib::ski_area::SkiArea;

use std::mem::take;

#[self_referencing]
struct LoadedRoute {
    ski_area: SkiArea,
    #[borrows(ski_area)]
    #[covariant]
    route: Result<AnalyzedRoute<'this>>,
}

impl LoadedRoute {
    fn create(ski_area: SkiArea, gpx: Gpx) -> Self {
        LoadedRouteBuilder {
            ski_area,
            route_builder: |ski_area| analyze_route(&ski_area, gpx),
        }
        .build()
    }

    fn get_route<'a>(&'a self) -> &'a AnalyzedRoute<'a> {
        self.borrow_route().as_ref().unwrap()
    }
}

#[derive(Default)]
enum LoadedData {
    #[default]
    None,
    SkiArea(SkiArea),
    Route(LoadedRoute),
}

#[derive(Default)]
pub struct AppState {
    loaded_data: LoadedData,
}

impl AppState {
    pub fn get_ski_area(&self) -> Option<&SkiArea> {
        match &self.loaded_data {
            LoadedData::None => None,
            LoadedData::SkiArea(s) => Some(s),
            LoadedData::Route(r) => Some(r.borrow_ski_area()),
        }
    }

    pub fn get_route<'a>(&'a self) -> Option<&AnalyzedRoute<'a>> {
        match &self.loaded_data {
            LoadedData::Route(r) => Some(r.get_route()),
            _ => None,
        }
    }

    pub fn set_ski_area(&mut self, ski_area: SkiArea) {
        self.loaded_data = LoadedData::SkiArea(ski_area);
    }

    pub fn set_gpx(&mut self, gpx: Gpx) -> Result<()> {
        let ski_area = match take(&mut self.loaded_data) {
            LoadedData::None => {
                Err(Error::new_s(ErrorType::LogicError, "No ski area loaded"))
            }
            LoadedData::SkiArea(s) => Ok(s),
            LoadedData::Route(r) => Ok(r.into_heads().ski_area),
        }?;
        let route = LoadedRoute::create(ski_area, gpx);
        route.borrow_route().as_ref().map_err(|e| e.clone())?;
        self.loaded_data = LoadedData::Route(route);
        Ok(())
    }
}
