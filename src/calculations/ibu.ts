import type {
  HopAdditionType,
  VolumeType,
  MassType,
} from "@beerjson/beerjson";
import type { Calculation, StaticCalculation } from "./types";
import {
  massToOunces,
  timeToMinutes,
  volumeToGallons,
} from "./units";

/**
 * Calculate milligrams per liter of added alpha acids
 */
function mgplAddedAlphaAcids(
  hop: HopAdditionType,
  batchSize: VolumeType
): number {
  const alphaAcid = hop.alpha_acid?.value || 0;
  return (
    ((alphaAcid / 100) * massToOunces(hop.amount as MassType) * 7490) /
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
  return (1 - Math.exp(-0.04 * timeToMinutes(hop.timing.time))) / 4.15;
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
