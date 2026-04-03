use serde::Serialize;
use specta::Type;

#[derive(Serialize, Type)]
#[serde(rename = "Point")]
pub struct PointDef {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Type)]
#[serde(rename = "Rect")]
pub struct RectDef {
    pub min: PointDef,
    pub max: PointDef,
}

#[derive(Serialize, Type)]
#[serde(rename = "LineString")]
pub struct LineStringDef(pub Vec<PointDef>);

#[derive(Serialize, Type)]
#[serde(rename = "MultiLineString")]
pub struct MultiLineStringDef(pub Vec<LineStringDef>);

#[derive(Serialize, Type)]
#[serde(rename = "Polygon")]
pub struct PolygonDef {
    pub exterior: Vec<PointDef>,
    pub interiors: Vec<Vec<PointDef>>,
}

#[derive(Serialize, Type)]
#[serde(rename = "MultiPolygon")]
pub struct MultiPolygonDef(pub Vec<PolygonDef>);
