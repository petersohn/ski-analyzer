import {
  Rect,
  LineString,
  BoundedGeometry,
  PointWithElevation,
  MultiPolygon,
  MultiLineString,
  Polygon,
} from "./geo";
import { indexData } from "@/utils/data";

export type SkiAreaMetadata = {
  id: number;
  name: string;
  outline: BoundedGeometry<Polygon>;
};

export type Lift = {
  ref: string;
  name: string;
  type: string;
  line: BoundedGeometry<LineString>;
  stations: PointWithElevation[];
  can_go_reverse: boolean;
  can_disembark: boolean;
  lengths: number[];
};

export type Difficulty =
  | ""
  | "Novice"
  | "Easy"
  | "Intermediate"
  | "Advanced"
  | "Expoert"
  | "Freeride";

export type Piste = {
  ref: string;
  name: string;
  difficulty: Difficulty;
  bounding_rect: Rect;
  areas: MultiPolygon;
  lines: MultiLineString;
};

export type RawSkiArea = {
  name: string;
  lifts: { [id: string]: Lift };
  pistes: { [id: string]: Piste };
  bounding_rect: Rect;
};

export type SkiArea = {
  name: string;
  lifts: Map<string, Lift>;
  pistes: Map<string, Piste>;
  bounding_rect: Rect;
};

export function indexSkiArea(ski_area: RawSkiArea): SkiArea {
  return {
    name: ski_area.name,
    lifts: indexData<Lift>(ski_area.lifts),
    pistes: indexData<Piste>(ski_area.pistes),
    bounding_rect: ski_area.bounding_rect,
  };
}
