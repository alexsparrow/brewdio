import { test, expect, describe } from "bun:test";
import {
  calculateAbv,
  calculateOg,
  calculateFg,
  calculateIbu,
  calculateColor,
  calculateWater,
  calculateCarbonation,
} from "brewdio-wasm";

import type {
  FermentableAdditionType,
  CultureAdditionType,
  HopAdditionType,
  VolumeType,
  PercentType,
  WaterCalculatorInput,
  CarbonationInput,
} from "brewdio-wasm";

// ============================================================================
// ABV
// ============================================================================

describe("calculateAbv", () => {
  test("standard beer", () => {
    const abv = calculateAbv(1.050, 1.010);
    expect(abv).toBeCloseTo(5.25, 1);
  });

  test("zero difference returns 0", () => {
    expect(calculateAbv(1.040, 1.040)).toBe(0);
  });
});

// ============================================================================
// OG
// ============================================================================

describe("calculateOg", () => {
  test("simple grain bill", () => {
    const fermentables: FermentableAdditionType[] = [
      {
        name: "Pale Malt",
        type: "grain",
        amount: { value: 10, unit: "lb" },
        yield: {
          fine_grind: { value: 80, unit: "%" },
        },
        color: { value: 3, unit: "SRM" },
      },
    ];
    const batch_size: VolumeType = { value: 5, unit: "gal" };
    const efficiency: PercentType = { value: 72, unit: "%" };

    const og = calculateOg(fermentables, batch_size, efficiency);
    expect(og).toBeGreaterThan(1.0);
    expect(og).toBeLessThan(1.1);
  });
});

// ============================================================================
// FG
// ============================================================================

describe("calculateFg", () => {
  test("with yeast attenuation", () => {
    const cultures: CultureAdditionType[] = [
      {
        name: "US-05",
        type: "ale",
        form: "dry",
        amount: { value: 1, unit: "pkg" },
        attenuation: { value: 77, unit: "%" },
      },
    ];

    const fg = calculateFg(1.050, cultures);
    expect(fg).toBeGreaterThan(1.0);
    expect(fg).toBeLessThan(1.050);
  });
});

// ============================================================================
// IBU
// ============================================================================

describe("calculateIbu", () => {
  test("single hop addition", () => {
    const hops: HopAdditionType[] = [
      {
        name: "Cascade",
        alpha_acid: { value: 5.5, unit: "%" },
        amount: { value: 1, unit: "oz" },
        timing: {
          time: { value: 60, unit: "min" },
          use: "add_to_boil",
        },
      },
    ];
    const batch_size: VolumeType = { value: 5, unit: "gal" };

    const ibu = calculateIbu(hops, batch_size, 1.050);
    expect(ibu).toBeGreaterThan(0);
    expect(ibu).toBeLessThan(100);
  });
});

// ============================================================================
// Color (SRM calculation)
// ============================================================================

describe("calculateColor", () => {
  test("pale grain bill", () => {
    const fermentables: FermentableAdditionType[] = [
      {
        name: "Pale Malt",
        type: "grain",
        amount: { value: 10, unit: "lb" },
        yield: {
          fine_grind: { value: 80, unit: "%" },
        },
        color: { value: 3, unit: "SRM" },
      },
    ];
    const batch_size: VolumeType = { value: 5, unit: "gal" };

    const srm = calculateColor(fermentables, batch_size);
    expect(srm).toBeGreaterThan(0);
    expect(srm).toBeLessThan(20);
  });
});

// ============================================================================
// Water Calculator
// ============================================================================

describe("calculateWater", () => {
  test("basic water calculation", () => {
    const input: WaterCalculatorInput = {
      target_batch_size: { value: 5, unit: "gal" },
      boil_time: { value: 60, unit: "min" },
      grain_bill: [
        {
          name: "Pale Malt",
          type: "grain",
          amount: { value: 10, unit: "lb" },
          yield: {
            fine_grind: { value: 80, unit: "%" },
          },
          color: { value: 3, unit: "SRM" },
        },
      ],
      mash_steps: [
        {
          name: "Mash In",
          type: "infusion",
          step_temperature: { value: 152, unit: "F" },
          step_time: { value: 60, unit: "min" },
        },
      ],
      grain_temperature: { value: 68, unit: "F" },
      equipment: undefined,
      units: undefined,
    };

    const result = calculateWater(input);
    expect(result).toBeDefined();
    expect(result.totalWaterNeeded).toBeDefined();
    expect(result.totalWaterNeeded.value).toBeGreaterThan(0);
    expect(result.strikeWater).toBeDefined();
    expect(result.strikeWater.volume.value).toBeGreaterThan(0);
    expect(result.strikeWater.temperature.value).toBeGreaterThan(0);
  });
});

// ============================================================================
// Carbonation Calculator
// ============================================================================

describe("calculateCarbonation", () => {
  test("basic carbonation calculation", () => {
    const input: CarbonationInput = {
      target_carbonation: { value: 2.5, unit: "vols" },
      beer_temperature: { value: 65, unit: "F" },
      beer_volume: { value: 5, unit: "gal" },
      units: undefined,
    };

    const result = calculateCarbonation(input);
    expect(result).toBeDefined();
    expect(result.priming_sugar).toBeDefined();
    expect(result.priming_sugar.corn_sugar.value).toBeGreaterThan(0);
    expect(result.priming_sugar.table_sugar.value).toBeGreaterThan(0);
    expect(result.priming_sugar.dme.value).toBeGreaterThan(0);
    expect(result.forced_carbonation).toBeDefined();
    expect(result.forced_carbonation.pressure.value).toBeGreaterThan(0);
  });
});
