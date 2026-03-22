use schemars::JsonSchema;

#[derive(JsonSchema)]
#[schemars(title = "Point")]
pub struct PointDef {
    pub x: f64,
    pub y: f64,
}

#[derive(JsonSchema)]
#[schemars(title = "Rect")]
pub struct RectDef {
    pub min: PointDef,
    pub max: PointDef,
}

#[derive(JsonSchema)]
#[schemars(title = "LineString")]
pub struct LineStringDef(pub Vec<PointDef>);

#[derive(JsonSchema)]
#[schemars(title = "MultiLineString")]
pub struct MultiLineStringDef(pub Vec<LineStringDef>);

#[derive(JsonSchema)]
#[schemars(title = "Polygon")]
pub struct PolygonDef {
    pub exterior: Vec<PointDef>,
    pub interiors: Vec<Vec<PointDef>>,
}

#[derive(JsonSchema)]
#[schemars(title = "MultiPolygon")]
pub struct MultiPolygonDef(pub Vec<PolygonDef>);
