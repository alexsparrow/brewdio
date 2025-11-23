import type {
  HopAdditionType,
  VolumeType,
  MassType,
  TimeType,
} from "@beerjson/beerjson";
import type { Calculation, StaticCalculation } from "./types";

/**
 * Convert amount to ounces
 */
function amountInOunces(amount: VolumeType | MassType): number {
  switch (amount.unit) {
    case "kg":
      return 35.274 * amount.value;
    case "g":
      return (35.275 * amount.value) / 1000;
    default:
      throw Error(`Don't know how to convert: ${amount.unit}`);
  }
}

/**
 * Convert time to minutes
 */
function timeInMinutes(time: TimeType): number {
  switch (time.unit) {
    case "min":
      return time.value;
    case "hr":
      return time.value * 60;
    default:
      throw Error(`Don't know how to convert: ${time.unit}`);
  }
}

/**
 * Convert volume to gallons
 */
function volumeToGallons(volume: VolumeType): number {
  switch (volume.unit) {
    case "gal":
      return volume.value;
    case "l":
      return volume.value * 0.264172;
    case "ml":
      return (volume.value / 1000) * 0.264172;
    default:
      throw Error(`Unrecognised volume unit: ${volume.unit}`);
  }
}

/**
 * Calculate milligrams per liter of added alpha acids
 */
function mgplAddedAlphaAcids(
  hop: HopAdditionType,
  batchSize: VolumeType
): number {
  const alphaAcid = hop.alpha_acid?.value || 0;
  return (
    ((alphaAcid / 100) * amountInOunces(hop.amount) * 7490) /
    volumeToGallons(batchSize)
  );
}

/**
 * Calculate boil time factor for hop utilization
 */
function boilTimeFactor(hop: HopAdditionType): number {
  if (!hop.timing?.time) {
    return 0;
  }
  return (1 - Math.exp(-0.04 * timeInMinutes(hop.timing.time))) / 4.15;
}

/**
 * IBU (International Bitterness Units) Calculation using Tinseth method
 * See: https://homebrewacademy.com/ibu-calculator/
 */
const ibuCalculation: Calculation<number> = {
  dependsOn: [
    "recipe.ingredients.hop_additions",
    "recipe.batch_size",
    "calculations.og",
  ] as any,
  function: (
    hops: HopAdditionType[],
    batchSize: VolumeType,
    og: number
  ) => {
    const hopAdditions = hops || [];

    if (hopAdditions.length === 0) {
      return 0;
    }

    // Bigness factor based on original gravity
    const bignessFactor = 1.65 * Math.pow(0.000125, og - 1);

    // Calculate IBU for each hop addition
    const ibus = hopAdditions.map((hop) => {
      const btf = boilTimeFactor(hop);
      const mgpl = mgplAddedAlphaAcids(hop, batchSize);
      const hopIbu = bignessFactor * btf * mgpl;
      return hopIbu;
    });

    const totalIbu = ibus.reduce((a, b) => a + b, 0);

    return totalIbu;
  },
};

// Export as StaticCalculation
export const IBU: StaticCalculation<number> = {
  type: "static",
  id: "ibu",
  calculation: ibuCalculation,
};
