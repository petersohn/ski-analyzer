import { describe, it, expect } from "vitest";
import { TrackConverter, type RawTrack } from "./track";
import { type SkiArea } from "./skiArea";

describe("TrackConverter", () => {
  const createMockSkiArea = (): SkiArea => ({
    name: "Test",
    lifts: new Map([
      [
        "lift1",
        {
          ref: "lift1",
          name: "Main Lift",
          type: "chairlift",
          line: {
            item: [],
            bounding_rect: {
              min: { x: 0, y: 0 },
              max: { x: 1, y: 1 },
            },
          },
          stations: [],
          can_go_reverse: false,
          can_disembark: false,
          lengths: [],
        },
      ],
    ]),
    pistes: new Map([
      [
        "piste1",
        {
          ref: "piste1",
          name: "Main Piste",
          difficulty: "Easy",
          bounding_rect: {
            min: { x: 0, y: 0 },
            max: { x: 2, y: 2 },
          },
          areas: [],
          lines: [],
        },
      ],
    ]),
    bounding_rect: {
      min: { x: 0, y: 0 },
      max: { x: 2, y: 2 },
    },
  });

  describe("convertTrack", () => {
    it("should convert a raw track with moving activity", () => {
      const skiArea = createMockSkiArea();
      const converter = new TrackConverter(skiArea);

      const rawTrack: RawTrack = {
        item: [
          {
            type: { Moving: { move_type: "ski", piste_id: "piste1" } },
            route: [
              [
                {
                  point: { x: 0, y: 0 },
                  time: "2024-01-01T10:00:00Z",
                  elevation: 1000,
                },
                {
                  point: { x: 1, y: 1 },
                  time: "2024-01-01T10:01:00Z",
                  elevation: 900,
                },
              ],
            ],
            begin_time: "2024-01-01T10:00:00Z",
            end_time: "2024-01-01T10:01:00Z",
            length: 100,
          },
        ],
        bounding_rect: {
          min: { x: 0, y: 0 },
          max: { x: 1, y: 1 },
        },
      };

      const result = converter.convertTrack(rawTrack);

      expect(result.item.length).toBe(1);
      expect(result.item[0].type).toBe("Moving");
      expect(result.item[0].moving?.move_type).toBe("ski");
      expect(result.item[0].moving?.piste?.name).toBe("Main Piste");
      expect(result.item[0].begin_time?.toISOString()).toBe(
        "2024-01-01T10:00:00.000Z",
      );
      expect(result.item[0].end_time?.toISOString()).toBe(
        "2024-01-01T10:01:00.000Z",
      );
    });

    it("should convert a track with use lift activity", () => {
      const skiArea = createMockSkiArea();
      const converter = new TrackConverter(skiArea);

      const rawTrack: RawTrack = {
        item: [
          {
            type: {
              UseLift: {
                lift_id: "lift1",
                begin_station: 0,
                end_station: 1,
                is_reverse: false,
              },
            },
            route: [],
            begin_time: "2024-01-01T10:00:00Z",
            end_time: "2024-01-01T10:05:00Z",
            length: 500,
          },
        ],
        bounding_rect: {
          min: { x: 0, y: 0 },
          max: { x: 1, y: 1 },
        },
      };

      const result = converter.convertTrack(rawTrack);

      expect(result.item[0].type).toBe("UseLift");
      expect(result.item[0].useLift?.lift.name).toBe("Main Lift");
      expect(result.item[0].useLift?.begin_station).toBe(0);
      expect(result.item[0].useLift?.end_station).toBe(1);
    });

    it("should handle unknown activity type", () => {
      const skiArea = createMockSkiArea();
      const converter = new TrackConverter(skiArea);

      const rawTrack: RawTrack = {
        item: [
          {
            type: { Unknown: null },
            route: [],
            begin_time: null,
            end_time: null,
            length: 0,
          },
        ],
        bounding_rect: {
          min: { x: 0, y: 0 },
          max: { x: 1, y: 1 },
        },
      };

      const result = converter.convertTrack(rawTrack);

      expect(result.item[0].type).toBe("Unknown");
    });

    it("should handle moving activity with empty piste id", () => {
      const skiArea = createMockSkiArea();
      const converter = new TrackConverter(skiArea);

      const rawTrack: RawTrack = {
        item: [
          {
            type: { Moving: { move_type: "ski", piste_id: "" } },
            route: [],
            begin_time: null,
            end_time: null,
            length: 0,
          },
        ],
        bounding_rect: {
          min: { x: 0, y: 0 },
          max: { x: 1, y: 1 },
        },
      };

      const result = converter.convertTrack(rawTrack);

      expect(result.item[0].type).toBe("Moving");
      expect(result.item[0].moving?.piste).toBeUndefined();
    });
  });
});
