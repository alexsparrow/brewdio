import type {
  FermentableAdditionType,
  VolumeType,
  MassType,
} from "@beerjson/beerjson";
import type { Calculation, StaticCalculation } from "./types";
import {
  srmToLovibond,
  colorToSrm,
  massToPounds,
  volumeToGallons,
} from "./units";

/**
 * Calculate MCU (Malt Color Units) for a fermentable
 */
function mcu(
  fermentable: FermentableAdditionType,
  batchSize: VolumeType
): number {
  // Check both fermentable.type.color and fermentable.color
  const colorData = (fermentable.type as any)?.color || (fermentable as any).color;

  if (!colorData) {
    return 0;
  }

  const srm = colorToSrm(colorData);
  const lovibond = srmToLovibond(srm);
  const massInPounds = massToPounds(fermentable.amount as MassType);
  const mcuValue = (massInPounds * lovibond) / volumeToGallons(batchSize);
  return mcuValue;
}

/**
 * Convert MCU to SRM using Morey equation
 */
function mcuToSrm(mcu: number): number {
  return 1.4922 * Math.pow(mcu, 0.6859);
}

/**
 * Beer Color Calculation (EBC)
 * Calculates the beer color based on fermentables and batch size
 */
const colorCalculation: Calculation<number> = {
  dependsOn: ["recipe.ingredients.fermentable_additions", "recipe.batch_size"],
  function: (
    fermentables: FermentableAdditionType[],
    batchSize: VolumeType
  ) => {
    const fermentableAdditions = fermentables || [];

    const totalMcu = fermentableAdditions
      .map((f) => mcu(f, batchSize))
      .reduce((a, b) => a + b, 0);

    const srm = mcuToSrm(totalMcu);

    return srm;
  },
};

// Export as StaticCalculation
export const Color: StaticCalculation<number> = {
  type: "static",
  id: "color",
  calculation: colorCalculation,
};
