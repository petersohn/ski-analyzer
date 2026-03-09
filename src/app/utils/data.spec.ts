import { describe, it, expect } from "vitest";
import { indexData, indexAndConvertData } from "./data";

describe("indexData", () => {
  it("should convert object to Map", () => {
    const input = { a: { id: 1 }, b: { id: 2 } };
    const result = indexData(input);
    expect(result).toBeInstanceOf(Map);
    expect(result.size).toBe(2);
    expect(result.get("a")).toEqual({ id: 1 });
    expect(result.get("b")).toEqual({ id: 2 });
  });

  it("should return empty Map for empty object", () => {
    const input = {};
    const result = indexData(input);
    expect(result).toBeInstanceOf(Map);
    expect(result.size).toBe(0);
  });
});

describe("indexAndConvertData", () => {
  it("should convert object to Map with transformation", () => {
    const input = { a: 1, b: 2 };
    const convert = (x: number) => x * 2;
    const result = indexAndConvertData(input, convert);
    expect(result).toBeInstanceOf(Map);
    expect(result.get("a")).toBe(2);
    expect(result.get("b")).toBe(4);
  });

  it("should return empty Map for empty object", () => {
    const input = {};
    const convert = (x: number) => x;
    const result = indexAndConvertData(input, convert);
    expect(result).toBeInstanceOf(Map);
    expect(result.size).toBe(0);
  });
});
