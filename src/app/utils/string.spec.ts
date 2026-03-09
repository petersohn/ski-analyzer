import { describe, it, expect } from "vitest";
import { filterString } from "./string";

describe("filterString", () => {
  it("should return true for empty search string", () => {
    expect(filterString("hello", "")).toBe(true);
  });

  it("should return true for exact match", () => {
    expect(filterString("hello", "hello")).toBe(true);
  });

  it("should return true for substring match at beginning", () => {
    expect(filterString("hello world", "hello")).toBe(true);
  });

  it("should return true for substring match in middle", () => {
    expect(filterString("hello world", "lo w")).toBe(true);
  });

  it("should return true for substring match at end", () => {
    expect(filterString("hello world", "world")).toBe(true);
  });

  it("should return false when search is longer than input", () => {
    expect(filterString("hi", "hello")).toBe(false);
  });

  it("should be case insensitive", () => {
    expect(filterString("Hello", "hello")).toBe(true);
    expect(filterString("HELLO", "hello")).toBe(true);
    expect(filterString("hello", "HELLO")).toBe(true);
  });

  it("should handle unicode characters", () => {
    expect(filterString("héllo", "héllo")).toBe(true);
    expect(filterString("Héllo", "héllo")).toBe(true);
  });

  it("should return false for non-matching string", () => {
    expect(filterString("hello", "xyz")).toBe(false);
  });
});
