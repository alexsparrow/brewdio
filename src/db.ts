import { createCollection } from "@tanstack/react-db";
import { dexieCollectionOptions } from "tanstack-dexie-db-collection";
import { z } from "zod";
import type { RecipeType, VolumeUnitType, MassUnitType, TemperatureUnitType, EquipmentType, BrewType } from "@beerjson/beerjson";

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

// Equipment schema that wraps BeerJSON EquipmentType with database fields
const equipmentSchema = z.object({
  id: z.string(),
  equipment: z.any() as z.ZodType<EquipmentType>, // BeerJSON equipment
  createdAt: z.number(),
  updatedAt: z.number(),
});

export type EquipmentDocument = z.infer<typeof equipmentSchema>;

export const equipmentCollection = createCollection(
  dexieCollectionOptions({
    id: "equipment",
    schema: equipmentSchema,
    getKey: (item) => item.id,
  })
);

// Default equipment profile matching current brewery structure
export const DEFAULT_EQUIPMENT: EquipmentDocument = {
  id: "default",
  equipment: {
    name: "Default Setup",
    equipment_items: [
      {
        name: "Mash Tun",
        form: "Mash Tun",
        maximum_volume: { value: 20, unit: "gal" },
        loss: { value: 0.25, unit: "gal" },
        grain_absorption_rate: { value: 0.96, unit: "l/kg" },
      },
      {
        name: "Boil Kettle",
        form: "Brew Kettle",
        maximum_volume: { value: 15, unit: "gal" },
        loss: { value: 0.5, unit: "gal" },
        boil_rate_per_hour: { value: 1.0, unit: "gal" },
      },
      {
        name: "Fermenter",
        form: "Fermenter",
        maximum_volume: { value: 10, unit: "gal" },
        loss: { value: 0.5, unit: "gal" },
      },
      {
        name: "Kegging",
        form: "Packaging Vessel",
        maximum_volume: { value: 5, unit: "gal" },
        loss: { value: 0, unit: "gal" },
      },
    ],
  },
  createdAt: Date.now(),
  updatedAt: Date.now(),
};

// Batch schema that includes recipe, equipment, and brew-specific data
const batchSchema = z.object({
  id: z.string(),
  name: z.string(),
  recipeId: z.string(), // Reference to original recipe
  equipmentId: z.string(), // Reference to equipment profile used
  recipe: z.any() as z.ZodType<RecipeType>, // Snapshot of recipe at brew time
  equipment: z.any() as z.ZodType<EquipmentType>, // Snapshot of equipment at brew time
  brewDate: z.number(), // Timestamp of brew start
  notes: z.string().optional(), // Batch-specific notes (separate from recipe notes)
  createdAt: z.number(),
  updatedAt: z.number(),
});

export type BatchDocument = z.infer<typeof batchSchema>;

export const batchesCollection = createCollection(
  dexieCollectionOptions({
    id: "batches",
    schema: batchSchema,
    getKey: (item) => item.id,
  })
);
