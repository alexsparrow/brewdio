import type {
  FermentableAdditionType,
  VolumeType,
  PercentType,
  YieldType,
  MassType,
} from "@beerjson/beerjson";
import type { Calculation, StaticCalculation } from "./types";

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
 * Convert amount to pounds
 */
function amountToLb(amount: MassType | VolumeType): number {
  switch (amount.unit) {
    case "kg":
      return amount.value * 2.20462;
    case "g":
      return (amount.value / 1000) * 2.20462;
    case "lb":
    case "lbs":
      return amount.value;
    case "oz":
      return amount.value / 16;
    default:
      throw Error(`Unrecognised unit: ${amount.unit}`);
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
 * Calculate gravity units for a fermentable
 */
function gravityUnit(fermentable: FermentableAdditionType): number {
  // Handle both fermentable.type.yield and fermentable.yield
  const yieldData = (fermentable.type as any)?.yield || (fermentable as any).yield;

  if (!yieldData) {
    return 0;
  }

  const yieldPPG = yieldToPPG(yieldData);
  const amountLb = amountToLb(fermentable.amount);
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
