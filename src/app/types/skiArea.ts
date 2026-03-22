import { indexData } from "@/utils/data";

// Import raw types from Rust (object dictionaries)
import type { SkiArea as RawSkiAreaType } from "./generated/skiArea";

// Import and re-export sub-types from their canonical locations
export type { Lift } from "./generated/lift";
export type { Piste } from "./generated/piste";
export type { SkiAreaMetadata } from "./generated/skiAreaMetadata";
export type { Rect } from "./generated/rect";
export type { Point } from "./generated/point";
export type { BoundedLineString } from "./generated/boundedLineString";
export type { PointWithElevation } from "./generated/pointWithElevation";
export type { BoundedPolygon } from "./generated/boundedPolygon";
export type { Polygon } from "./generated/polygon";
export { Difficulty } from "./generated/difficulty";

import type { Lift } from "./generated/lift";
import type { Piste } from "./generated/piste";
import type { SkiAreaMetadata } from "./generated/skiAreaMetadata";
import type { Rect } from "./generated/rect";

// Re-export geo types for convenience
import type { Point } from "./generated/point";
import type { Polygon } from "./generated/polygon";
export type LineString = Point[];
export type MultiLineString = LineString[];
export type MultiPolygon = Polygon[];

// RawSkiArea is the type received from Rust backend (with object dictionaries)
export type RawSkiArea = RawSkiAreaType;

// SkiArea is the type used in the app (with Maps for easier access)
export type SkiArea = {
  metadata: SkiAreaMetadata;
  lifts: Map<string, Lift>;
  pistes: Map<string, Piste>;
  bounding_rect: Rect;
  date: string;
};

export function indexSkiArea(ski_area: RawSkiArea): SkiArea {
  return {
    metadata: ski_area.metadata,
    lifts: indexData<Lift>(ski_area.lifts),
    pistes: indexData<Piste>(ski_area.pistes),
    bounding_rect: ski_area.bounding_rect,
    date: ski_area.date,
  };
}
