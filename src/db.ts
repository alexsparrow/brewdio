import { createCollection } from "@tanstack/react-db";
import { dexieCollectionOptions } from "tanstack-dexie-db-collection";
import { z } from "zod";
import type { RecipeType, VolumeUnitType, MassUnitType, TemperatureUnitType } from "@beerjson/beerjson";

const todoSchema = z.object({
  id: z.string(),
  text: z.string(),
  completed: z.boolean(),
});

export const todosCollection = createCollection(
  dexieCollectionOptions({
    id: "todos",
    schema: todoSchema,
    getKey: (item) => item.id,
  })
);

// Recipe schema that wraps BeerJSON RecipeType with database fields
const recipeSchema = z.object({
  id: z.string(),
  recipe: z.any() as z.ZodType<RecipeType>, // BeerJSON recipe
  createdAt: z.number(),
  updatedAt: z.number(),
});

export type RecipeDocument = z.infer<typeof recipeSchema>;

export const recipesCollection = createCollection(
  dexieCollectionOptions({
    id: "recipes",
    schema: recipeSchema,
    getKey: (item) => item.id,
  })
);

// Settings schema
const settingsSchema = z.object({
  id: z.string(),
  vimMode: z.boolean(),
  defaultVolumeUnit: z.enum(["ml", "l", "tsp", "tbsp", "floz", "cup", "pt", "qt", "gal", "bbl"]),
  defaultMassUnit: z.enum(["mg", "g", "kg", "lb", "oz"]),
  defaultTemperatureUnit: z.enum(["C", "F"]),
  openaiApiKey: z.string().optional(),
  defaultAuthor: z.string().optional(),
});

export type SettingsDocument = z.infer<typeof settingsSchema>;

export const settingsCollection = createCollection(
  dexieCollectionOptions({
    id: "settings",
    schema: settingsSchema,
    getKey: (item) => item.id,
  })
);

// Default settings
export const DEFAULT_SETTINGS: SettingsDocument = {
  id: "user-settings",
  vimMode: false,
  defaultVolumeUnit: "gal",
  defaultMassUnit: "lb",
  defaultTemperatureUnit: "F",
  openaiApiKey: "",
  defaultAuthor: "Brewdio User",
};
