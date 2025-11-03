import { Store, Derived } from "@tanstack/store";
import type { FermentableType, RecipeType } from "@beerjson/beerjson";

// import { Parser } from 'expr-eval'

interface RuntimeState {
  fermentables: FermentableType[];
  recipe: RecipeType;
}

interface CalcDef {
  id: string;
  expr: string; // user-authored expression
  dependsOn: string[]; // e.g. ["recipe.og", "values.fermentables_summary"]
}

interface Stores {
  state: Store<RuntimeState>;
  calculations: Map<string, Store<any>>;
}

// Create the global store
export const stores: Stores = {
  state: new Store<RuntimeState>({
    fermentables: [],
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

export function wireCalculations(stores: Stores, defs: CalcDef[]): Unsub[] {
  console.log("wiring");
  const unsubs: Unsub[] = [];

  for (const def of defs) {
    stores.calculations.set(def.id, new Store(undefined));
  }

  const deriveds = new Map();

  for (const def of defs) {
    const fn = new Function(
      ...def.dependsOn.map((_, i) => `arg${i}`),
      `return (${def.expr})(...arguments)`
    );

    // Build a selector that extracts only the dependent fields
    const deps = def.dependsOn.map((path) => {
      if (path.startsWith("calculations.")) {
        const key = path.split(".")[1];
        return stores.calculations.get(key)!;
      }

      return stores.state;
    });

    const derived = new Derived({
      deps,
      fn: (...args) => {
        const processedArgs = def.dependsOn.map((dep, idx) => {
          const path = dep.startsWith("calculations.")
            ? dep.split(".").slice(1)
            : dep.split(".");

          if (dep.startsWith("calculations.")) {
            return args[0].currDepVals[idx];
          }

          return path.reduce(
            (obj, k) => (obj as any)?.[k],
            args[0].currDepVals[idx]
          ) as any;
        });
        const value = fn(...processedArgs);
        console.log("returning", def.id, value);
        return value;
      },
    });

    const unsub = derived.subscribe((v) => {
      console.log("updating", def.id, v.currentVal);
      stores.calculations.get(def.id)?.setState(() => v.currentVal);
    });

    const unmount = derived.mount();
    unsubs.push(() => {
      unsub();
      unmount();
    });

    deriveds.set(def.id, derived);
  }

  for (const def of defs) {
    const currentVal = deriveds.get(def.id).state;
    console.log("initialising", def.id, currentVal);
    stores.calculations.get(def.id)?.setState(() => currentVal);
  }

  return unsubs;
}
