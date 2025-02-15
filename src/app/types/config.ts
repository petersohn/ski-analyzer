import { Point } from "./geo";
import { SkiAreaMetadata } from "./skiArea";
import { Dayjs } from "dayjs";
import dayjs from "dayjs";

export type MapConfig = {
  center: Point;
  zoom: number;
};

export type RawCachedSkiArea = {
  uuid: string;
  metadata: SkiAreaMetadata;
  date: string;
};

export type CachedSkiArea = {
  uuid: string;
  metadata: SkiAreaMetadata;
  date: Dayjs;
};

export type MapTileType = "OpenStreetMap" | "Custom";

export type UiConfig = {
  mapTileType: MapTileType;
  mapTileUrl: string;
};

export function convertCachedSkiAreas(
  input: RawCachedSkiArea[],
): CachedSkiArea[] {
  return input.map((data) => {
    return {
      uuid: data.uuid,
      metadata: data.metadata,
      date: dayjs(data.date),
    };
  });
}
