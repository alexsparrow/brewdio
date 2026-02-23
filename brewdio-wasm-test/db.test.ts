import { test, expect, describe, beforeEach } from "bun:test";
import { RecipeDb } from "brewdio-wasm";

import type { RecipeType } from "brewdio-wasm";

function sampleRecipe(name: string = "Test IPA"): RecipeType {
  return {
    name,
    type: "all grain",
    author: "Tester",
    batch_size: { unit: "l", value: 20.0 },
    efficiency: {
      brewhouse: { unit: "%", value: 72.0 },
    },
    ingredients: {
      fermentable_additions: [
        {
          name: "Pale Malt",
          type: "grain",
          amount: { unit: "kg", value: 5.0 },
          color: { unit: "SRM", value: 2.0 },
          yield: { fine_grind: { unit: "%", value: 80.0 } },
        },
      ],
      hop_additions: [
        {
          name: "Cascade",
          timing: {
            duration: { unit: "min", value: 60 },
            use: "add_to_boil",
          },
          amount: { unit: "g", value: 30.0 },
          form: "pellet",
          alpha_acid: { unit: "%", value: 5.5 },
        },
      ],
    },
  } as RecipeType;
}

describe("RecipeDb", () => {
  let db: RecipeDb;

  beforeEach(() => {
    db = new RecipeDb();
  });

  describe("create", () => {
    test("returns a recipe ID", () => {
      const id = db.create_recipe("My IPA", sampleRecipe());
      expect(typeof id).toBe("string");
      expect(id.length).toBeGreaterThan(0);
    });

    test("creates unique IDs", () => {
      const id1 = db.create_recipe("Beer 1", sampleRecipe("Beer 1"));
      const id2 = db.create_recipe("Beer 2", sampleRecipe("Beer 2"));
      expect(id1).not.toBe(id2);
    });
  });

  describe("get", () => {
    test("returns the created recipe", () => {
      const id = db.create_recipe("Get Test IPA", sampleRecipe("Get Test IPA"));
      const result = db.get_recipe(id);
      expect(result).not.toBeNull();
      expect(result.id).toBe(id);
      expect(result.name).toBe("Get Test IPA");
      expect(result.recipe).toBeDefined();
      expect(result.recipe.name).toBe("Get Test IPA");
    });

    test("returns null for nonexistent ID", () => {
      const result = db.get_recipe("nonexistent-id");
      expect(result).toBeNull();
    });

    test("recipe data survives round-trip", () => {
      const recipe = sampleRecipe("Roundtrip Beer");
      const id = db.create_recipe("Roundtrip Beer", recipe);
      const result = db.get_recipe(id);

      expect(result.recipe.batch_size.value).toBe(20.0);
      expect(result.recipe.batch_size.unit).toBe("l");
      expect(result.recipe.efficiency.brewhouse.value).toBe(72.0);
      expect(result.recipe.ingredients.fermentable_additions).toHaveLength(1);
      expect(result.recipe.ingredients.fermentable_additions[0].name).toBe("Pale Malt");
      expect(result.recipe.ingredients.hop_additions).toHaveLength(1);
      expect(result.recipe.ingredients.hop_additions[0].name).toBe("Cascade");
    });
  });

  describe("list", () => {
    test("empty database returns empty array", () => {
      const recipes = db.list_recipes();
      expect(Array.isArray(recipes)).toBe(true);
      expect(recipes).toHaveLength(0);
    });

    test("lists created recipes", () => {
      db.create_recipe("Beer A", sampleRecipe("Beer A"));
      db.create_recipe("Beer B", sampleRecipe("Beer B"));

      const recipes = db.list_recipes();
      expect(recipes).toHaveLength(2);
      const names = recipes.map((r: any) => r.name).sort();
      expect(names).toEqual(["Beer A", "Beer B"]);
    });

    test("does not include deleted recipes", () => {
      const id = db.create_recipe("Soon Deleted", sampleRecipe("Soon Deleted"));
      db.delete_recipe(id);

      const recipes = db.list_recipes();
      expect(recipes).toHaveLength(0);
    });
  });

  describe("update", () => {
    test("updates recipe name", () => {
      const id = db.create_recipe("Original", sampleRecipe("Original"));
      const updated = sampleRecipe("Updated");
      db.update_recipe(id, "Updated", updated);

      const result = db.get_recipe(id);
      expect(result.name).toBe("Updated");
      expect(result.recipe.name).toBe("Updated");
    });

    test("updates recipe data", () => {
      const id = db.create_recipe("Batch Test", sampleRecipe("Batch Test"));
      const updated = sampleRecipe("Batch Test");
      updated.batch_size = { unit: "gal", value: 5.0 } as any;
      db.update_recipe(id, "Batch Test", updated);

      const result = db.get_recipe(id);
      expect(result.recipe.batch_size.unit).toBe("gal");
      expect(result.recipe.batch_size.value).toBe(5.0);
    });

    test("preserves ID after update", () => {
      const id = db.create_recipe("ID Test", sampleRecipe("ID Test"));
      db.update_recipe(id, "ID Test v2", sampleRecipe("ID Test v2"));

      const result = db.get_recipe(id);
      expect(result.id).toBe(id);
    });
  });

  describe("delete", () => {
    test("soft-deletes a recipe", () => {
      const id = db.create_recipe("Delete Me", sampleRecipe("Delete Me"));
      db.delete_recipe(id);

      const result = db.get_recipe(id);
      expect(result).toBeNull();
    });

    test("does not affect other recipes", () => {
      const id1 = db.create_recipe("Keep", sampleRecipe("Keep"));
      const id2 = db.create_recipe("Remove", sampleRecipe("Remove"));
      db.delete_recipe(id2);

      const kept = db.get_recipe(id1);
      expect(kept).not.toBeNull();
      expect(kept.name).toBe("Keep");

      const recipes = db.list_recipes();
      expect(recipes).toHaveLength(1);
    });
  });

  describe("change notification", () => {
    test("fires callback on create", () => {
      let called = false;
      db.on_recipes_change(() => { called = true; });
      db.create_recipe("Notify Test", sampleRecipe("Notify Test"));
      expect(called).toBe(true);
    });

    test("fires callback on update", () => {
      const id = db.create_recipe("Update Notify", sampleRecipe("Update Notify"));
      let called = false;
      db.on_recipes_change(() => { called = true; });
      db.update_recipe(id, "Updated", sampleRecipe("Updated"));
      expect(called).toBe(true);
    });

    test("fires callback on delete", () => {
      const id = db.create_recipe("Delete Notify", sampleRecipe("Delete Notify"));
      let called = false;
      db.on_recipes_change(() => { called = true; });
      db.delete_recipe(id);
      expect(called).toBe(true);
    });
  });
});
