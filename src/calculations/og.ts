import type {
  FermentableAdditionType,
  VolumeType,
  PercentType,
  YieldType,
  MassType,
} from "@beerjson/beerjson";
import type { Calculation, StaticCalculation } from "./types";
import {
  massToPounds,
  volumeToGallons,
} from "./units";

/**
 * Convert yield to Points Per Gallon (PPG)
 */
function yieldToPPG(yield_: YieldType): number {
  if (yield_.fine_grind) {
    switch (yield_.fine_grind.unit) {
      case "%":
        return (yield_.fine_grind.value / 100) * 46.21;
    }
  }
  throw Error("Don't know how to handle this yield");
}

/**
 * Calculate gravity units for a fermentable
 */
function gravityUnit(fermentable: FermentableAdditionType): number {
  // Handle both fermentable.type.yield and fermentable.yield
  const yieldData = (fermentable.type as any)?.yield || (fermentable as any).yield;

  if (!yieldData) {
    return 0;
  }

  const yieldPPG = yieldToPPG(yieldData);
  const amountLb = massToPounds(fermentable.amount as MassType);
  return yieldPPG * amountLb;
}

/**
 * Original Gravity (OG) Calculation
 * Calculates the original gravity based on fermentables, batch size, and efficiency
 */
const ogCalculation: Calculation<number> = {
  dependsOn: [
    "recipe.ingredients.fermentable_additions",
    "recipe.batch_size",
    "recipe.efficiency.brewhouse",
  ],
  function: (
    fermentables: FermentableAdditionType[],
    batchSize: VolumeType,
    efficiency: PercentType
  ) => {
    const fermentableAdditions = fermentables || [];
    const efficiencyValue = efficiency?.value || 70; // default 70%

    // Calculate total gravity units
    const totalGravity = fermentableAdditions
      .map((f) => gravityUnit(f))
      .reduce((a, b) => a + b, 0);

    // Apply efficiency
    const totalGravityWithEfficiency = (totalGravity * efficiencyValue) / 100;

    // Adjust for post-boil volume
    const postBoilVolumeInGallons = volumeToGallons(batchSize) + 0.53;

    // Calculate final OG
    const gravityUnits = totalGravityWithEfficiency / postBoilVolumeInGallons;
    const og = 1 + gravityUnits / 1000;

    return og;
  },
};

// Export as StaticCalculation
export const OG: StaticCalculation<number> = {
  type: "static",
  id: "og",
  calculation: ogCalculation,
};
