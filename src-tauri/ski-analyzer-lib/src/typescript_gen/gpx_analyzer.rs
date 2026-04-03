use serde::Serialize;
use specta::Type;

use super::geo::RectDef;
use crate::gpx_analyzer::Activity;

#[derive(Serialize, Type)]
#[serde(rename = "AnalyzedRoute")]
pub struct AnalyzedRouteDef {
    pub item: Vec<Activity>,
    pub bounding_rect: RectDef,
}
