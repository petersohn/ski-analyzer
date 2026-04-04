import { Rect } from "./geo";
import type { Lift, Piste, SkiAreaMetadata } from "./generated/generated";
import { indexData } from "@/utils/data";
import { SkiArea as RawSkiArea } from "./generated/generated";

export {
  PointWithElevation,
  Difficulty,
  Lift,
  PisteMetadata,
  PisteData,
  Piste,
  SkiAreaMetadata,
} from "./generated/generated";

export { RawSkiArea };

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
