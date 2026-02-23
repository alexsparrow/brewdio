import { useStore } from "@tanstack/react-store";
import { stores } from "@/lib/calculate";
import { useSyncExternalStore } from "react";

/**
 * React hook to access a calculated value from the calculation store.
 *
 * This hook automatically subscribes to changes in the calculation and
 * triggers re-renders when the calculated value updates.
 *
 * @example
 * ```tsx
 * function RecipeComponent() {
 *   const og = useCalculation<number>("og");
 *   const abv = useCalculation<number>("abv");
 *
 *   return (
 *     <div>
 *       <p>OG: {og?.toFixed(3)}</p>
 *       <p>ABV: {abv?.toFixed(1)}%</p>
 *     </div>
 *   );
 * }
 * ```
 *
 * @param calculationId - The ID of the calculation to retrieve (e.g., "og", "fg", "abv")
 * @returns The calculated value, or undefined if not yet available
 */
export function useCalculation<T = any>(calculationId: string): T | undefined {
  const value = useSyncExternalStore(
    // Subscribe function - called by React to set up subscription
    (callback) => {
      const calculationStore = stores.calculations.get(calculationId);

      if (!calculationStore) {
        return () => {}; // no-op unsubscribe for non-existent calculations
      }

      // Subscribe to store changes and notify React when value updates
      return calculationStore.subscribe(() => {
        callback();
      });
    },
    // Snapshot function - called by React to get current value
    () => {
      const calculationStore = stores.calculations.get(calculationId);
      const val = calculationStore?.state as T | undefined;
      return val;
    }
  );

  return value;
}
