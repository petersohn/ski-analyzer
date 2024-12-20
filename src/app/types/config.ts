import { Point } from "./geo";
import { SkiAreaMetadata } from "./skiArea";
import { Dayjs } from "dayjs";
import dayjs from "dayjs";
import { indexAndConvertData } from "@/utils/data";

export type MapConfig = {
  center: Point;
  zoom: number;
};

export type RawCachedSkiArea = {
  metadata: SkiAreaMetadata;
  date: string;
};

export type RawCachedSkiAreas = { [uuid: string]: RawCachedSkiArea };

export type CachedSkiArea = {
  metadata: SkiAreaMetadata;
  date: Dayjs;
};

export type CachedSkiAreas = Map<string, CachedSkiArea>;

export function convertCachedSkiAreas(
  input: RawCachedSkiAreas,
): CachedSkiAreas {
  return indexAndConvertData(input, (x) => {
    return { metadata: x.metadata, date: dayjs(x.date) };
  });
}
