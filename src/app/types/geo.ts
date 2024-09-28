export type Point = {
  x: number;
  y: number;
};

export type LineString = Point[];

export type MultiLineString = LineString[];

export type Polygon = {
  exterior: LineString;
  interiors: MultiLineString;
};

export type MultiPolygon = Polygon[];

export type Rect = {
  min: Point;
  max: Point;
};

export type BoundedGeometry<T> = {
  item: T;
  bounding_rect: Rect;
};

export type PointWithElevation = {
  point: Point;
  elevation: number;
};
