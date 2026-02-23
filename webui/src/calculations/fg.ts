import type { Calculation, StaticCalculation } from "./types";

/**
 * Final Gravity (FG) Calculation
 * Calculates the final gravity based on OG and yeast attenuation
 */
const fgCalculation: Calculation<number> = {
  dependsOn: ["calculations.og", "recipe.ingredients.culture_additions"] as any,
  function: (og: number, cultures: any[]) => {
    // Get attenuation from first yeast, or use default (75%)
    let attenuationValue = 0.75;

    if (cultures && cultures.length > 0 && cultures[0].attenuation) {
      attenuationValue = cultures[0].attenuation.value / 100;
    }

    // FG = OG - (OG - 1) × attenuation
    const fg = og - (og - 1) * attenuationValue;

    return fg;
  },
};

// Export as StaticCalculation
export const FG: StaticCalculation<number> = {
  type: "static",
  id: "fg",
  calculation: fgCalculation,
};
