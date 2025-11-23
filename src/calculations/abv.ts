import type { Calculation, StaticCalculation } from "./types";

/**
 * Alcohol By Volume (ABV) Calculation
 * Calculates ABV based on OG and FG
 */
const abvCalculation: Calculation<number> = {
  dependsOn: ["calculations.og", "calculations.fg"] as any,
  function: (og: number, fg: number) => {
    // ABV = (OG - FG) × 131.25
    const abv = (og - fg) * 131.25;

    return abv;
  },
};

// Export as StaticCalculation
export const ABV: StaticCalculation<number> = {
  type: "static",
  id: "abv",
  calculation: abvCalculation,
};
