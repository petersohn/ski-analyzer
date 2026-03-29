use serde::Serialize;
use specta::Type;

use super::geo::RectDef;

#[derive(Serialize, Type)]
#[serde(rename = "BoundedGeometry")]
pub struct BoundedGeometryDef<T: Type> {
    pub item: T,
    pub bounding_rect: RectDef,
}
