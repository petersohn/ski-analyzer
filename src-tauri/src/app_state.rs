use ski_analyzer_lib::ski_area::SkiArea;

#[derive(Default)]
pub struct AppState {
    pub active_ski_area: Option<SkiArea>,
}
