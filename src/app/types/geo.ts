// Re-export geo types from generated files
export { Point } from "./generated/point";
export { Rect } from "./generated/rect";
export { PointWithElevation } from "./generated/pointWithElevation";

import type { Point } from "./generated/point";

// LineString is an array of points
export type LineString = Point[];

// MultiLineString is an array of LineStrings
export type MultiLineString = LineString[];

// Polygon with proper array types
import type { Polygon } from "./generated/polygon";
export type { Polygon };

// MultiPolygon is an array of Polygons
export type MultiPolygon = Polygon[];

// BoundedGeometry wraps an item with its bounding rect
import type { Rect } from "./generated/rect";
export type BoundedGeometry<T> = {
  bounding_rect: Rect;
  item: T;
};
