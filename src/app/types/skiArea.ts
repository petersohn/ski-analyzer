import {
  Rect,
  LineString,
  BoundedGeometry,
  PointWithElevation,
  MultiPolygon,
  MultiLineString,
} from "./geo";

export type Lift = {
  unique_id: string;
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
  unique_id: string;
  ref: string;
  name: string;
  difficulty: Difficulty;
  bounding_rect: Rect;
  areas: MultiPolygon;
  lines: MultiLineString;
};

export type RawSkiArea = {
  name: string;
  lifts: Lift[];
  pistes: Piste[];
  bounding_rect: Rect;
};

export type SkiArea = {
  name: string;
  lifts: Map<string, Lift>;
  pistes: Map<string, Piste>;
  bounding_rect: Rect;
};

function indexData<T extends { unique_id: string }>(data: T[]): Map<string, T> {
  return new Map(data.map((x) => [x.unique_id, x]));
}

export function index_ski_area(ski_area: RawSkiArea): SkiArea {
  return {
    name: ski_area.name,
    lifts: indexData<Lift>(ski_area.lifts),
    pistes: indexData<Piste>(ski_area.pistes),
    bounding_rect: ski_area.bounding_rect,
  };
}
