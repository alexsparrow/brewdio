import type { RecipeType } from "@beerjson/beerjson";
import type { Path } from "@clickbar/dot-diver";

/**
 * The runtime state available to all calculations.
 * This represents the current recipe being edited.
 */
export interface RuntimeState {
  recipe: RecipeType;
}

/**
 * A calculation definition with dependencies and a function.
 * Dependencies are type-safe paths into the RuntimeState or other calculations.
 */
export interface Calculation<T> {
  /** Type-safe paths to dependencies (e.g., "recipe.batch_size" or "calculations.og") */
  dependsOn: Path<RuntimeState>[];
  /** The calculation function that receives dependency values as arguments */
  function: (...args: any[]) => T;
}

/**
 * A static calculation defined in TypeScript code.
 * These calculations are pre-compiled and type-safe.
 *
 * Example:
 * ```ts
 * export const OG: StaticCalculation<number> = {
 *   type: "static",
 *   id: "og",
 *   calculation: {
 *     dependsOn: ["recipe.ingredients.fermentable_additions", "recipe.batch_size"],
 *     function: (fermentables, batchSize) => { ... }
 *   }
 * };
 * ```
 */
export interface StaticCalculation<T = any> {
  type: "static";
  id: string;
  calculation: Calculation<T>;
}

/**
 * A dynamic calculation defined as a string expression.
 * These calculations are evaluated at runtime using new Function().
 *
 * Example:
 * ```ts
 * {
 *   type: "dynamic",
 *   id: "total_weight",
 *   dependsOn: ["recipe.ingredients.fermentable_additions"],
 *   expr: "(fermentables) => fermentables.reduce((sum, f) => sum + f.amount.value, 0)"
 * }
 * ```
 */
export interface DynamicCalculation {
  type: "dynamic";
  id: string;
  /** String expression that will be evaluated as a function */
  expr: string;
  /** Dependency paths (same format as static calculations) */
  dependsOn: string[];
}

/**
 * Union type for all calculation definitions.
 * Calculations can be either static (TypeScript) or dynamic (runtime-evaluated).
 */
export type CalcDef = DynamicCalculation | StaticCalculation;
