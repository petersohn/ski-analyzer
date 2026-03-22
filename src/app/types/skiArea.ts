export {
  PointWithElevation,
  Difficulty,
  Lift,
  PisteMetadata,
  PisteData,
  Piste,
  SkiAreaMetadata,
  SkiArea as RawSkiArea,
} from "./generated/skiArea";

import { Rect } from "./geo";
import type { Lift, Piste, SkiAreaMetadata } from "./generated/skiArea";
import { indexData } from "@/utils/data";

export type SkiArea = {
  metadata: SkiAreaMetadata;
  lifts: Map<string, Lift>;
  pistes: Map<string, Piste>;
  bounding_rect: Rect;
  date: string;
};

export function indexSkiArea(
  ski_area: import("./generated/skiArea").SkiArea,
): SkiArea {
  return {
    metadata: ski_area.metadata,
    lifts: indexData<Lift>(ski_area.lifts),
    pistes: indexData<Piste>(ski_area.pistes),
    bounding_rect: ski_area.bounding_rect,
    date: ski_area.date,
  };
}
