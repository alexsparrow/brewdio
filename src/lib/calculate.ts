/**
 * Reactive Calculation System
 *
 * This module implements a reactive calculation graph for brewing recipe calculations.
 *
 * ## Architecture
 *
 * The system uses TanStack Store to create a dependency graph where:
 * - Each calculation is a Derived value that automatically recalculates when dependencies change
 * - Calculations can depend on recipe state (e.g., fermentables, batch size) or other calculations
 * - The graph ensures calculations execute in the correct order and propagate changes correctly
 *
 * ## Key Components
 *
 * 1. **State Store**: Holds the current recipe data
 * 2. **Calculation Stores**: Individual stores for each calculation result (OG, FG, ABV, etc.)
 * 3. **Derived Values**: Reactive computations that automatically update when dependencies change
 * 4. **Dependency Graph**: Calculations that depend on other calculations receive updates via Derived subscriptions
 *
 * ## Critical Implementation Details
 *
 * - Calculations that depend on other calculations must subscribe to the **Derived** object,
 *   not the calculation's Store. This ensures proper reactivity and prevents race conditions.
 * - Calculations must be defined in dependency order (e.g., OG before FG, FG before ABV)
 * - Each calculation store is initialized immediately after its Derived is mounted to ensure
 *   dependent calculations can access the initial value
 *
 * @module calculate
 */

import { Store, Derived } from "@tanstack/store";
import type { FermentableType, RecipeType } from "@beerjson/beerjson";
import type { Path } from "@clickbar/dot-diver";
import type {
  CalcDef,
  DynamicCalculation,
  StaticCalculation,
  RuntimeState as CalcRuntimeState,
} from "../calculations/types";

interface RuntimeState {
  recipe: RecipeType;
}

interface Stores {
  state: Store<RuntimeState>;
  calculations: Map<string, Store<any>>;
}

// Create the global store
export const stores: Stores = {
  state: new Store<RuntimeState>({
    recipe: {
      name: "",
      type: "cider",
      author: "",
      batch_size: {
        unit: "gal",
        value: 5,
      },
      efficiency: {
        conversion: undefined,
        lauter: undefined,
        mash: undefined,
        brewhouse: {
          unit: "%",
          value: 0,
        },
      },
      ingredients: {
        fermentable_additions: [],
        hop_additions: undefined,
        miscellaneous_additions: undefined,
        culture_additions: undefined,
        water_additions: undefined,
      },
    },
  }),
  calculations: new Map(),
};

type Unsub = () => void;

/**
 * Wire up calculations to create a reactive calculation graph.
 *
 * This function creates a dependency graph where:
 * 1. Each calculation is represented as a TanStack Store Derived value
 * 2. Calculations can depend on recipe state or other calculations
 * 3. When dependencies change, calculations automatically recalculate
 * 4. Results are stored in calculation stores for React components to consume
 *
 * Important: Calculations are processed in order, so calculations that depend
 * on other calculations must be defined after their dependencies in the array.
 *
 * @param stores - The stores object containing state and calculation stores
 * @param defs - Array of calculation definitions (static or dynamic)
 * @returns Array of unsubscribe functions to clean up subscriptions
 */
export function wireCalculations(stores: Stores, defs: CalcDef[]): Unsub[] {
  const unsubs: Unsub[] = [];

  // Step 1: Create empty stores for all calculations
  // These stores will be populated as calculations complete
  for (const def of defs) {
    stores.calculations.set(def.id, new Store(undefined));
  }

  // Map to store Derived objects for inter-calculation dependencies
  const deriveds = new Map();

  // Step 2: Wire up each calculation
  for (const def of defs) {
    let fn: (...args: any[]) => any;
    let dependsOn: readonly (string | Path<CalcRuntimeState>)[];

    // Create the calculation function based on type
    if (def.type === "static") {
      // Static: Use pre-defined TypeScript function
      fn = def.calculation.function;
      dependsOn = def.calculation.dependsOn;
    } else {
      // Dynamic: Evaluate string expression using new Function()
      fn = new Function(
        ...def.dependsOn.map((_, i) => `arg${i}`),
        `return (${def.expr})(...arguments)`
      );
      dependsOn = def.dependsOn;
    }

    // Step 3: Build dependency array for TanStack Derived
    // For each dependency path, determine if it's a calculation or state dependency
    const deps = dependsOn.map((path) => {
      if (typeof path === "string" && path.startsWith("calculations.")) {
        const key = path.split(".")[1];
        const calcDerived = deriveds.get(key);
        if (calcDerived) {
          // Depend on the Derived object directly for proper reactivity
          // This ensures that when the upstream calculation updates,
          // this calculation will automatically recalculate
          return calcDerived;
        }
        // Fallback to store (shouldn't happen if definitions are in order)
        return stores.calculations.get(key)!;
      }

      // Depend on the main state store for recipe data
      return stores.state;
    });

    // Step 4: Create the Derived value
    const derived = new Derived({
      deps,
      fn: (...args) => {
        // Extract values from dependencies
        const processedArgs = dependsOn.map((dep, idx) => {
          const depStr = String(dep);
          const path = depStr.startsWith("calculations.")
            ? depStr.split(".").slice(1)
            : depStr.split(".");

          if (depStr.startsWith("calculations.")) {
            // For calculation dependencies, the value is already extracted
            return args[0].currDepVals[idx];
          }

          // For state dependencies, navigate the path to extract the value
          const val = path.reduce(
            (obj, k) => (obj as any)?.[k],
            args[0].currDepVals[idx]
          ) as any;
          return val;
        });

        // Call the calculation function with extracted values
        return fn(...processedArgs);
      },
    });

    // Step 5: Subscribe to derived changes and update the calculation store
    // This makes the calculated value available to React components via useCalculation
    const unsub = derived.subscribe((v) => {
      stores.calculations.get(def.id)?.setState(() => v.currentVal);
    });

    // Step 6: Mount the derived to start tracking changes
    const unmount = derived.mount();

    // Step 7: Initialize the store with the current value immediately
    // This is critical for dependent calculations to have access to initial values
    const currentVal = derived.state;
    stores.calculations.get(def.id)?.setState(() => currentVal);

    // Track cleanup functions
    unsubs.push(() => {
      unsub();
      unmount();
    });

    // Store the Derived for use as a dependency by later calculations
    deriveds.set(def.id, derived);
  }

  return unsubs;
}
