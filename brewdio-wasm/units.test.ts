import { test, expect, describe } from "bun:test";
import {
  // Volume
  volume_to_gallons,
  volume_to_liters,
  volume_to_milliliters,
  // Mass
  mass_to_grams,
  mass_to_kilograms,
  mass_to_pounds,
  mass_to_ounces,
  // Time
  time_to_minutes,
  time_to_hours,
  // Temperature
  temperature_to_celsius,
  temperature_to_fahrenheit,
  // Color
  color_to_srm,
  srm_to_ebc,
  ebc_to_srm,
  lovibond_to_srm,
  srm_to_lovibond,
  // Gravity
  gravity_to_sg,
  gravity_to_plato,
  // Pressure
  pressure_to_psi,
  pressure_to_bar,
  pressure_to_kilopascals,
  // Misc
  percent_to_decimal,
  bitterness_to_ibu,
  carbonation_to_volumes,
} from "brewdio-wasm";

// ============================================================================
// Volume
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
// Mass
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
// Time
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
// Temperature
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
// Color
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
// Gravity
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
// Pressure
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
// Percent, Bitterness, Carbonation
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
