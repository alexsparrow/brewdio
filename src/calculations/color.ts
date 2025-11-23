import type {
  FermentableAdditionType,
  VolumeType,
  ColorType,
} from "@beerjson/beerjson";
import type { Calculation, StaticCalculation } from "./types";

/**
 * Convert SRM to EBC
 */
function srmToEbc(srm: number): number {
  return 1.97 * srm;
}

/**
 * Convert EBC to SRM
 */
function ebcToSrm(ebc: number): number {
  return ebc / 1.97;
}

/**
 * Convert Lovibond to SRM
 */
function lovibondToSrm(lovibond: number): number {
  return 1.3546 * lovibond - 0.76;
}

/**
 * Convert SRM to Lovibond
 */
function srmToLovibond(srm: number): number {
  return (srm + 0.76) / 1.3546;
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
 * Convert color type to SRM
 */
function colorToSrm(colorType: ColorType): number {
  switch (colorType.unit) {
    case "EBC":
      return ebcToSrm(colorType.value);
    case "SRM":
      return colorType.value;
    case "Lovi":
      return lovibondToSrm(colorType.value);
    default:
      return 0;
  }
}

/**
 * Convert mass to pounds
 */
function massToLb(amount: FermentableAdditionType["amount"]): number {
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
      throw Error(`Unrecognised mass unit: ${amount.unit}`);
  }
}

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
  const massInPounds = massToLb(fermentable.amount);
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
    const ebc = srmToEbc(srm);

    return ebc;
  },
};

// Export as StaticCalculation
export const Color: StaticCalculation<number> = {
  type: "static",
  id: "color",
  calculation: colorCalculation,
};
