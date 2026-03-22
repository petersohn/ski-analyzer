import { indexData } from "@/utils/data";

// Re-export all types from generated schema
export {
  SkiArea as RawSkiArea,
  LiftValue as Lift,
  PisteValue as Piste,
  Metadata as SkiAreaMetadata,
  Difficulty,
  BoundingRect as Rect,
  Max as Point,
  Line as BoundedLineString,
  StationElement as PointWithElevation,
  Outline as BoundedPolygon,
  Item as Polygon,
} from "./generated/skiArea";

// Import types for transformation
import type {
  SkiArea as RawSkiAreaType,
  LiftValue,
  PisteValue,
  Metadata,
  BoundingRect,
} from "./generated/skiArea";

// SkiArea is the transformed type with Maps for easier access
export type SkiArea = {
  metadata: Metadata;
  lifts: Map<string, LiftValue>;
  pistes: Map<string, PisteValue>;
  bounding_rect: BoundingRect;
  date: string;
};

export function indexSkiArea(ski_area: RawSkiAreaType): SkiArea {
  return {
    metadata: ski_area.metadata,
    lifts: indexData<LiftValue>(ski_area.lifts),
    pistes: indexData<PisteValue>(ski_area.pistes),
    bounding_rect: ski_area.bounding_rect,
    date: ski_area.date,
  };
}
