import {
  Rect,
  LineString,
  BoundedGeometry,
  PointWithElevation,
  MultiPolygon,
  MultiLineString,
} from "./geo";

export type Lift = {
  ref: string;
  name: string;
  type: string;
  line: BoundedGeometry<LineString>;
  stations: PointWithElevation[];
  can_go_reverse: boolean;
  can_disembark: boolean;
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

export type SkiArea = {
  name: string;
  lifts: Lift[];
  pistes: Piste[];
  bounding_rect: Rect;
};
