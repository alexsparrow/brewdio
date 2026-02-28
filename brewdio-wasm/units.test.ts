import { test, expect, describe } from "bun:test";
import {
  // Volume
  volumeToGallons,
  volumeToLiters,
  volumeToMilliliters,
  // Mass
  massToGrams,
  massToKilograms,
  massToPounds,
  massToOunces,
  // Time
  timeToMinutes,
  timeToHours,
  // Temperature
  temperatureToCelsius,
  temperatureToFahrenheit,
  // Color
  colorToSrm,
  srmToEbc,
  ebcToSrm,
  lovibondToSrm,
  srmToLovibond,
  // Gravity
  gravityToSg,
  gravityToPlato,
  // Pressure
  pressureToPsi,
  pressureToBar,
  pressureToKilopascals,
  // Misc
  percentToDecimal,
  bitternessToIbu,
  carbonationToVolumes,
} from "brewdio-wasm";

// ============================================================================
// Volume
// ============================================================================

describe("volume conversions", () => {
  test("gallons to gallons", () => {
    expect(volumeToGallons({ value: 5, unit: "gal" })).toBeCloseTo(5, 5);
  });

  test("liters to gallons", () => {
    expect(volumeToGallons({ value: 18.927, unit: "l" })).toBeCloseTo(5, 1);
  });

  test("gallons to liters", () => {
    expect(volumeToLiters({ value: 1, unit: "gal" })).toBeCloseTo(3.78541, 2);
  });

  test("liters to milliliters", () => {
    expect(volumeToMilliliters({ value: 1, unit: "l" })).toBeCloseTo(1000, 0);
  });
});

// ============================================================================
// Mass
// ============================================================================

describe("mass conversions", () => {
  test("pounds to grams", () => {
    expect(massToGrams({ value: 1, unit: "lb" })).toBeCloseTo(453.592, 0);
  });

  test("kilograms to pounds", () => {
    expect(massToPounds({ value: 1, unit: "kg" })).toBeCloseTo(2.20462, 2);
  });

  test("ounces to grams", () => {
    expect(massToGrams({ value: 1, unit: "oz" })).toBeCloseTo(28.3495, 0);
  });

  test("grams to kilograms", () => {
    expect(massToKilograms({ value: 1000, unit: "g" })).toBeCloseTo(1, 5);
  });

  test("pounds to ounces", () => {
    expect(massToOunces({ value: 1, unit: "lb" })).toBeCloseTo(16, 1);
  });
});

// ============================================================================
// Time
// ============================================================================

describe("time conversions", () => {
  test("hours to minutes", () => {
    expect(timeToMinutes({ value: 1, unit: "hr" })).toBeCloseTo(60, 5);
  });

  test("minutes to hours", () => {
    expect(timeToHours({ value: 90, unit: "min" })).toBeCloseTo(1.5, 5);
  });
});

// ============================================================================
// Temperature
// ============================================================================

describe("temperature conversions", () => {
  test("celsius to fahrenheit", () => {
    expect(temperatureToFahrenheit({ value: 100, unit: "C" })).toBeCloseTo(212, 1);
  });

  test("fahrenheit to celsius", () => {
    expect(temperatureToCelsius({ value: 212, unit: "F" })).toBeCloseTo(100, 1);
  });

  test("freezing point", () => {
    expect(temperatureToFahrenheit({ value: 0, unit: "C" })).toBeCloseTo(32, 1);
  });
});

// ============================================================================
// Color
// ============================================================================

describe("color conversions", () => {
  test("colorToSrm from SRM", () => {
    expect(colorToSrm({ value: 10, unit: "SRM" })).toBeCloseTo(10, 5);
  });

  test("colorToSrm from EBC", () => {
    // EBC = SRM * 1.97
    expect(colorToSrm({ value: 19.7, unit: "EBC" })).toBeCloseTo(10, 0);
  });

  test("srmToEbc", () => {
    expect(srmToEbc(10)).toBeCloseTo(19.7, 0);
  });

  test("ebcToSrm", () => {
    expect(ebcToSrm(19.7)).toBeCloseTo(10, 0);
  });

  test("lovibondToSrm", () => {
    const srm = lovibondToSrm(10);
    expect(srm).toBeGreaterThan(0);
  });

  test("srmToLovibond roundtrip", () => {
    const srm = lovibondToSrm(10);
    const lov = srmToLovibond(srm);
    expect(lov).toBeCloseTo(10, 1);
  });
});

// ============================================================================
// Gravity
// ============================================================================

describe("gravity conversions", () => {
  test("sg to sg", () => {
    expect(gravityToSg({ value: 1.050, unit: "sg" })).toBeCloseTo(1.050, 5);
  });

  test("plato to sg", () => {
    // 12 Plato ≈ 1.048
    expect(gravityToSg({ value: 12, unit: "plato" })).toBeCloseTo(1.048, 2);
  });

  test("sg to plato", () => {
    expect(gravityToPlato({ value: 1.048, unit: "sg" })).toBeCloseTo(12, 0);
  });
});

// ============================================================================
// Pressure
// ============================================================================

describe("pressure conversions", () => {
  test("psi to psi", () => {
    expect(pressureToPsi({ value: 14.696, unit: "psi" })).toBeCloseTo(14.696, 3);
  });

  test("bar to psi", () => {
    expect(pressureToPsi({ value: 1, unit: "bar" })).toBeCloseTo(14.5038, 1);
  });

  test("kPa to bar", () => {
    expect(pressureToBar({ value: 100, unit: "kPa" })).toBeCloseTo(1, 2);
  });

  test("psi to kPa", () => {
    expect(pressureToKilopascals({ value: 14.696, unit: "psi" })).toBeCloseTo(101.325, 0);
  });
});

// ============================================================================
// Percent, Bitterness, Carbonation
// ============================================================================

describe("misc conversions", () => {
  test("percentToDecimal", () => {
    expect(percentToDecimal({ value: 75, unit: "%" })).toBeCloseTo(0.75, 5);
  });

  test("bitternessToIbu", () => {
    expect(bitternessToIbu({ value: 40, unit: "IBUs" })).toBeCloseTo(40, 5);
  });

  test("carbonationToVolumes", () => {
    expect(carbonationToVolumes({ value: 2.5, unit: "vols" })).toBeCloseTo(2.5, 5);
  });
});
