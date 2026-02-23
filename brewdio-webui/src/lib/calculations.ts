import { calculate_og, calculate_fg, calculate_abv, calculate_ibu, calculate_color } from "brewdio-wasm";
import type { StaticCalculation } from "./calculations-types";

export const OG: StaticCalculation<number> = {
  type: "static",
  id: "og",
  calculation: {
    dependsOn: ["recipe.ingredients.fermentable_additions", "recipe.batch_size", "recipe.efficiency.brewhouse"],
    function: (fermentables, batchSize, efficiency) =>
      calculate_og(fermentables || [], batchSize, efficiency || { value: 70, unit: "%" }),
  },
};

export const FG: StaticCalculation<number> = {
  type: "static",
  id: "fg",
  calculation: {
    dependsOn: ["calculations.og", "recipe.ingredients.culture_additions"] as any,
    function: (og, cultures) => calculate_fg(og, cultures || []),
  },
};

export const ABV: StaticCalculation<number> = {
  type: "static",
  id: "abv",
  calculation: {
    dependsOn: ["calculations.og", "calculations.fg"] as any,
    function: (og, fg) => calculate_abv(og, fg),
  },
};

export const IBU: StaticCalculation<number> = {
  type: "static",
  id: "ibu",
  calculation: {
    dependsOn: ["recipe.ingredients.hop_additions", "recipe.batch_size", "calculations.og"] as any,
    function: (hops, batchSize, og) => calculate_ibu(hops || [], batchSize, og),
  },
};

export const Color: StaticCalculation<number> = {
  type: "static",
  id: "color",
  calculation: {
    dependsOn: ["recipe.ingredients.fermentable_additions", "recipe.batch_size"],
    function: (fermentables, batchSize) => calculate_color(fermentables || [], batchSize),
  },
};
