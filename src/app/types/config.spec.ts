import { describe, it, expect } from "vitest";
import { convertCachedSkiAreas, type RawCachedSkiArea } from "./config";

describe("convertCachedSkiAreas", () => {
  it("should convert raw cached ski areas to formatted ones", () => {
    const rawData: RawCachedSkiArea[] = [
      {
        uuid: "uuid-1",
        metadata: {
          id: 1,
          name: "Ski Area 1",
          outline: {
            item: {
              exterior: [],
              interiors: [],
            },
            bounding_rect: {
              min: { x: 0, y: 0 },
              max: { x: 1, y: 1 },
            },
          },
        },
        date: "2024-01-15",
      },
      {
        uuid: "uuid-2",
        metadata: {
          id: 2,
          name: "Ski Area 2",
          outline: {
            item: {
              exterior: [],
              interiors: [],
            },
            bounding_rect: {
              min: { x: 0, y: 0 },
              max: { x: 1, y: 1 },
            },
          },
        },
        date: "2024-02-20",
      },
    ];

    const result = convertCachedSkiAreas(rawData);

    expect(result.length).toBe(2);
    expect(result[0].uuid).toBe("uuid-1");
    expect(result[0].date.format("YYYY-MM-DD")).toBe("2024-01-15");
    expect(result[1].uuid).toBe("uuid-2");
    expect(result[1].date.format("YYYY-MM-DD")).toBe("2024-02-20");
  });

  it("should handle empty array", () => {
    const result = convertCachedSkiAreas([]);
    expect(result.length).toBe(0);
  });
});
