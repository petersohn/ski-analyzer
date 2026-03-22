import { describe, it, expect } from "vitest";
import { indexSkiArea, type RawSkiArea, Difficulty } from "./skiArea";

describe("indexSkiArea", () => {
  it("should convert raw ski area to indexed ski area", () => {
    const rawSkiArea: RawSkiArea = {
      metadata: {
        id: 12345,
        name: "Test Ski Area",
        outline: {
          item: {
            exterior: [
              { x: 0, y: 0 },
              { x: 1, y: 0 },
              { x: 1, y: 1 },
              { x: 0, y: 1 },
              { x: 0, y: 0 },
            ],
            interiors: [],
          },
          bounding_rect: {
            min: { x: 0, y: 0 },
            max: { x: 1, y: 1 },
          },
        },
      },
      lifts: {
        lift1: {
          ref: "lift1",
          name: "Lift 1",
          type: "chairlift",
          line: {
            item: [
              { x: 0, y: 0 },
              { x: 1, y: 1 },
            ],
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
      },
      pistes: {
        piste1: {
          ref: "piste1",
          name: "Piste 1",
          difficulty: Difficulty.Easy,
          bounding_rect: {
            min: { x: 0, y: 0 },
            max: { x: 2, y: 2 },
          },
          areas: [],
          lines: [],
        },
      },
      bounding_rect: {
        min: { x: 0, y: 0 },
        max: { x: 2, y: 2 },
      },
      date: "2024-01-01T00:00:00Z",
    };

    const result = indexSkiArea(rawSkiArea);

    expect(result.metadata.name).toBe("Test Ski Area");
    expect(result.lifts.size).toBe(1);
    expect(result.lifts.get("lift1")?.name).toBe("Lift 1");
    expect(result.pistes.size).toBe(1);
    expect(result.pistes.get("piste1")?.name).toBe("Piste 1");
  });

  it("should handle empty lifts and pistes", () => {
    const rawSkiArea: RawSkiArea = {
      metadata: {
        id: 12345,
        name: "Empty Ski Area",
        outline: {
          item: {
            exterior: [
              { x: 0, y: 0 },
              { x: 1, y: 0 },
              { x: 1, y: 1 },
              { x: 0, y: 1 },
              { x: 0, y: 0 },
            ],
            interiors: [],
          },
          bounding_rect: {
            min: { x: 0, y: 0 },
            max: { x: 1, y: 1 },
          },
        },
      },
      lifts: {},
      pistes: {},
      bounding_rect: {
        min: { x: 0, y: 0 },
        max: { x: 1, y: 1 },
      },
      date: "2024-01-01T00:00:00Z",
    };

    const result = indexSkiArea(rawSkiArea);

    expect(result.lifts.size).toBe(0);
    expect(result.pistes.size).toBe(0);
  });
});
