use schemars::JsonSchema;

#[derive(JsonSchema)]
pub struct PointDef {
    pub x: f64,
    pub y: f64,
}

#[derive(JsonSchema)]
pub struct RectDef {
    pub min: PointDef,
    pub max: PointDef,
}

#[derive(JsonSchema)]
pub struct LineStringDef(pub Vec<PointDef>);

#[derive(JsonSchema)]
pub struct PolygonDef {
    pub exterior: Vec<PointDef>,
    pub interiors: Vec<Vec<PointDef>>,
}
