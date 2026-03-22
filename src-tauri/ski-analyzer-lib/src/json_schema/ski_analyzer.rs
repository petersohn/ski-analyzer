use super::geo::RectDef;
use schemars::JsonSchema;

#[derive(JsonSchema)]
pub struct BoundedGeometryDef<T>
where
    T: JsonSchema,
{
    pub item: T,
    pub bounding_rect: RectDef,
}
