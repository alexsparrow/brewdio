import { test, expect, describe } from "bun:test";
import { srmToSrgb, ebcToSrgb, rgbToHex } from "brewdio-wasm";

// ============================================================================
// Color Rendering (SRM → sRGB)
// ============================================================================

describe("color rendering", () => {
  test("srmToSrgb returns [r, g, b] array with values 0-1", () => {
    const rgb = srmToSrgb(5) as [number, number, number];
    expect(Array.isArray(rgb)).toBe(true);
    expect(rgb).toHaveLength(3);
    expect(rgb[0]).toBeGreaterThanOrEqual(0);
    expect(rgb[0]).toBeLessThanOrEqual(1);
    expect(rgb[1]).toBeGreaterThanOrEqual(0);
    expect(rgb[2]).toBeGreaterThanOrEqual(0);
  });

  test("ebcToSrgb returns [r, g, b] array with values 0-1", () => {
    const rgb = ebcToSrgb(10) as [number, number, number];
    expect(Array.isArray(rgb)).toBe(true);
    expect(rgb).toHaveLength(3);
    expect(rgb[0]).toBeGreaterThanOrEqual(0);
    expect(rgb[0]).toBeLessThanOrEqual(1);
  });

  test("darker beer has lower rgb values", () => {
    const light = srmToSrgb(5) as [number, number, number];
    const dark = srmToSrgb(40) as [number, number, number];
    // Darker beer should have lower green channel
    expect(dark[1]).toBeLessThan(light[1]);
  });

  test("rgbToHex", () => {
    expect(rgbToHex(255, 0, 0)).toBe("#ff0000");
    expect(rgbToHex(0, 255, 0)).toBe("#00ff00");
    expect(rgbToHex(0, 0, 255)).toBe("#0000ff");
  });
});
