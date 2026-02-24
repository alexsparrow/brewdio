import { test, expect, describe } from "bun:test";
import { srm_to_srgb, ebc_to_srgb, rgb_to_hex } from "brewdio-wasm";

// ============================================================================
// Color Rendering (SRM → sRGB)
// ============================================================================

describe("color rendering", () => {
  test("srm_to_srgb returns [r, g, b] array with values 0-1", () => {
    const rgb = srm_to_srgb(5) as [number, number, number];
    expect(Array.isArray(rgb)).toBe(true);
    expect(rgb).toHaveLength(3);
    expect(rgb[0]).toBeGreaterThanOrEqual(0);
    expect(rgb[0]).toBeLessThanOrEqual(1);
    expect(rgb[1]).toBeGreaterThanOrEqual(0);
    expect(rgb[2]).toBeGreaterThanOrEqual(0);
  });

  test("ebc_to_srgb returns [r, g, b] array with values 0-1", () => {
    const rgb = ebc_to_srgb(10) as [number, number, number];
    expect(Array.isArray(rgb)).toBe(true);
    expect(rgb).toHaveLength(3);
    expect(rgb[0]).toBeGreaterThanOrEqual(0);
    expect(rgb[0]).toBeLessThanOrEqual(1);
  });

  test("darker beer has lower rgb values", () => {
    const light = srm_to_srgb(5) as [number, number, number];
    const dark = srm_to_srgb(40) as [number, number, number];
    // Darker beer should have lower green channel
    expect(dark[1]).toBeLessThan(light[1]);
  });

  test("rgb_to_hex", () => {
    expect(rgb_to_hex(255, 0, 0)).toBe("#ff0000");
    expect(rgb_to_hex(0, 255, 0)).toBe("#00ff00");
    expect(rgb_to_hex(0, 0, 255)).toBe("#0000ff");
  });
});
