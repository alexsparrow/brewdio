import { test, expect, describe } from "bun:test";
import {
  calculate_abv,
  calculate_og,
  calculate_fg,
  calculate_ibu,
  calculate_color,
  calculate_water,
  calculate_carbonation,
  // Unit conversions
  volume_to_gallons,
  volume_to_liters,
  volume_to_milliliters,
  mass_to_grams,
  mass_to_kilograms,
  mass_to_pounds,
  mass_to_ounces,
  time_to_minutes,
  time_to_hours,
  temperature_to_celsius,
  temperature_to_fahrenheit,
  color_to_srm,
  srm_to_ebc,
  ebc_to_srm,
  lovibond_to_srm,
  srm_to_lovibond,
  gravity_to_sg,
  gravity_to_plato,
  pressure_to_psi,
  pressure_to_bar,
  pressure_to_kilopascals,
  percent_to_decimal,
  bitterness_to_ibu,
  carbonation_to_volumes,
  // Color rendering
  srm_to_srgb,
  ebc_to_srgb,
  rgb_to_hex,
} from "brewdio-wasm";

import type {
  FermentableAdditionType,
  CultureAdditionType,
  HopAdditionType,
  MashStepType,
  VolumeType,
  PercentType,
  WaterCalculatorInput,
  CarbonationInput,
} from "brewdio-wasm";

// ============================================================================
// ABV
// ============================================================================

describe("calculate_abv", () => {
  test("standard beer", () => {
    const abv = calculate_abv(1.050, 1.010);
    expect(abv).toBeCloseTo(5.25, 1);
  });

  test("zero difference returns 0", () => {
    expect(calculate_abv(1.040, 1.040)).toBe(0);
  });
});

// ============================================================================
// OG
// ============================================================================

describe("calculate_og", () => {
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

    const og = calculate_og(fermentables, batch_size, efficiency);
    expect(og).toBeGreaterThan(1.0);
    expect(og).toBeLessThan(1.1);
  });
});

// ============================================================================
// FG
// ============================================================================

describe("calculate_fg", () => {
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

    const fg = calculate_fg(1.050, cultures);
    expect(fg).toBeGreaterThan(1.0);
    expect(fg).toBeLessThan(1.050);
  });
});

// ============================================================================
// IBU
// ============================================================================

describe("calculate_ibu", () => {
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

    const ibu = calculate_ibu(hops, batch_size, 1.050);
    expect(ibu).toBeGreaterThan(0);
    expect(ibu).toBeLessThan(100);
  });
});

// ============================================================================
// Color
// ============================================================================

describe("calculate_color", () => {
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

    const srm = calculate_color(fermentables, batch_size);
    expect(srm).toBeGreaterThan(0);
    expect(srm).toBeLessThan(20);
  });
});

// ============================================================================
// Unit Conversions — Volume
// ============================================================================

describe("volume conversions", () => {
  test("gallons to gallons", () => {
    expect(volume_to_gallons({ value: 5, unit: "gal" })).toBeCloseTo(5, 5);
  });

  test("liters to gallons", () => {
    expect(volume_to_gallons({ value: 18.927, unit: "l" })).toBeCloseTo(5, 1);
  });

  test("gallons to liters", () => {
    expect(volume_to_liters({ value: 1, unit: "gal" })).toBeCloseTo(3.78541, 2);
  });

  test("liters to milliliters", () => {
    expect(volume_to_milliliters({ value: 1, unit: "l" })).toBeCloseTo(1000, 0);
  });
});

// ============================================================================
// Unit Conversions — Mass
// ============================================================================

describe("mass conversions", () => {
  test("pounds to grams", () => {
    expect(mass_to_grams({ value: 1, unit: "lb" })).toBeCloseTo(453.592, 0);
  });

  test("kilograms to pounds", () => {
    expect(mass_to_pounds({ value: 1, unit: "kg" })).toBeCloseTo(2.20462, 2);
  });

  test("ounces to grams", () => {
    expect(mass_to_grams({ value: 1, unit: "oz" })).toBeCloseTo(28.3495, 0);
  });

  test("grams to kilograms", () => {
    expect(mass_to_kilograms({ value: 1000, unit: "g" })).toBeCloseTo(1, 5);
  });

  test("pounds to ounces", () => {
    expect(mass_to_ounces({ value: 1, unit: "lb" })).toBeCloseTo(16, 1);
  });
});

// ============================================================================
// Unit Conversions — Time
// ============================================================================

describe("time conversions", () => {
  test("hours to minutes", () => {
    expect(time_to_minutes({ value: 1, unit: "hr" })).toBeCloseTo(60, 5);
  });

  test("minutes to hours", () => {
    expect(time_to_hours({ value: 90, unit: "min" })).toBeCloseTo(1.5, 5);
  });
});

// ============================================================================
// Unit Conversions — Temperature
// ============================================================================

describe("temperature conversions", () => {
  test("celsius to fahrenheit", () => {
    expect(temperature_to_fahrenheit({ value: 100, unit: "C" })).toBeCloseTo(212, 1);
  });

  test("fahrenheit to celsius", () => {
    expect(temperature_to_celsius({ value: 212, unit: "F" })).toBeCloseTo(100, 1);
  });

  test("freezing point", () => {
    expect(temperature_to_fahrenheit({ value: 0, unit: "C" })).toBeCloseTo(32, 1);
  });
});

// ============================================================================
// Unit Conversions — Color
// ============================================================================

describe("color conversions", () => {
  test("color_to_srm from SRM", () => {
    expect(color_to_srm({ value: 10, unit: "SRM" })).toBeCloseTo(10, 5);
  });

  test("color_to_srm from EBC", () => {
    // EBC = SRM * 1.97
    expect(color_to_srm({ value: 19.7, unit: "EBC" })).toBeCloseTo(10, 0);
  });

  test("srm_to_ebc", () => {
    expect(srm_to_ebc(10)).toBeCloseTo(19.7, 0);
  });

  test("ebc_to_srm", () => {
    expect(ebc_to_srm(19.7)).toBeCloseTo(10, 0);
  });

  test("lovibond_to_srm", () => {
    const srm = lovibond_to_srm(10);
    expect(srm).toBeGreaterThan(0);
  });

  test("srm_to_lovibond roundtrip", () => {
    const srm = lovibond_to_srm(10);
    const lov = srm_to_lovibond(srm);
    expect(lov).toBeCloseTo(10, 1);
  });
});

// ============================================================================
// Unit Conversions — Gravity
// ============================================================================

describe("gravity conversions", () => {
  test("sg to sg", () => {
    expect(gravity_to_sg({ value: 1.050, unit: "sg" })).toBeCloseTo(1.050, 5);
  });

  test("plato to sg", () => {
    // 12 Plato ≈ 1.048
    expect(gravity_to_sg({ value: 12, unit: "plato" })).toBeCloseTo(1.048, 2);
  });

  test("sg to plato", () => {
    expect(gravity_to_plato({ value: 1.048, unit: "sg" })).toBeCloseTo(12, 0);
  });
});

// ============================================================================
// Unit Conversions — Pressure
// ============================================================================

describe("pressure conversions", () => {
  test("psi to psi", () => {
    expect(pressure_to_psi({ value: 14.696, unit: "psi" })).toBeCloseTo(14.696, 3);
  });

  test("bar to psi", () => {
    expect(pressure_to_psi({ value: 1, unit: "bar" })).toBeCloseTo(14.5038, 1);
  });

  test("kPa to bar", () => {
    expect(pressure_to_bar({ value: 100, unit: "kPa" })).toBeCloseTo(1, 2);
  });

  test("psi to kPa", () => {
    expect(pressure_to_kilopascals({ value: 14.696, unit: "psi" })).toBeCloseTo(101.325, 0);
  });
});

// ============================================================================
// Unit Conversions — Percent, Bitterness, Carbonation
// ============================================================================

describe("misc conversions", () => {
  test("percent_to_decimal", () => {
    expect(percent_to_decimal({ value: 75, unit: "%" })).toBeCloseTo(0.75, 5);
  });

  test("bitterness_to_ibu", () => {
    expect(bitterness_to_ibu({ value: 40, unit: "IBUs" })).toBeCloseTo(40, 5);
  });

  test("carbonation_to_volumes", () => {
    expect(carbonation_to_volumes({ value: 2.5, unit: "vols" })).toBeCloseTo(2.5, 5);
  });
});

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

// ============================================================================
// Water Calculator
// ============================================================================

describe("calculate_water", () => {
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

    const result = calculate_water(input);
    expect(result).toBeDefined();
    expect(result.total_water_needed).toBeDefined();
    expect(result.total_water_needed.value).toBeGreaterThan(0);
    expect(result.strike_water).toBeDefined();
    expect(result.strike_water.volume.value).toBeGreaterThan(0);
    expect(result.strike_water.temperature.value).toBeGreaterThan(0);
  });
});

// ============================================================================
// Carbonation Calculator
// ============================================================================

describe("calculate_carbonation", () => {
  test("basic carbonation calculation", () => {
    const input: CarbonationInput = {
      target_carbonation: { value: 2.5, unit: "vols" },
      beer_temperature: { value: 65, unit: "F" },
      beer_volume: { value: 5, unit: "gal" },
      units: undefined,
    };

    const result = calculate_carbonation(input);
    expect(result).toBeDefined();
    expect(result.priming_sugar).toBeDefined();
    expect(result.priming_sugar.corn_sugar.value).toBeGreaterThan(0);
    expect(result.priming_sugar.table_sugar.value).toBeGreaterThan(0);
    expect(result.priming_sugar.dme.value).toBeGreaterThan(0);
    expect(result.forced_carbonation).toBeDefined();
    expect(result.forced_carbonation.pressure.value).toBeGreaterThan(0);
  });
});
