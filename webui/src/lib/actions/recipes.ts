import { recipesCollection, type RecipeDocument, mashProfilesCollection, createDefaultMashProfile, settingsCollection } from '@/db';
import type { RecipeType, StyleType } from '@beerjson/beerjson';
import styles from '@/data/styles.json';

export interface CreateRecipeParams {
  name: string;
  batchSize: number;
  batchSizeUnit?: 'gal' | 'l';
  styleName: string;
  author?: string;
}

export interface UpdateRecipeParams {
  recipeId: string;
  recipe: RecipeType;
}

/**
 * Create a new recipe with basic BeerJSON structure
 */
export async function createRecipe(params: CreateRecipeParams): Promise<string> {
  const { name, batchSize, batchSizeUnit = 'gal', styleName, author = 'Brewdio User' } = params;

  // Find the selected style from the styles data
  const selectedStyle = styles.find((s) => s.name === styleName);

  if (!selectedStyle) {
    throw new Error(`Style "${styleName}" not found`);
  }

  // Fetch user settings to get temperature preference
  const userSettings = await settingsCollection
    .getAll()
    .then((settings) => settings.find((s) => s.id === "user-settings"));

  const temperatureUnit = userSettings?.defaultTemperatureUnit || "F";

  // Fetch default mash profile or create one with user's temperature unit
  let defaultMash = await mashProfilesCollection
    .getAll()
    .then((profiles) =>
      profiles.find((p) => p.id === "default-single-infusion")
    );

  // If no default exists yet, create one
  if (!defaultMash) {
    defaultMash = createDefaultMashProfile(temperatureUnit);
  }

  // Create a basic BeerJSON recipe
  const newRecipe: RecipeType = {
    name,
    type: 'all grain',
    author,
    batch_size: {
      value: batchSize,
      unit: batchSizeUnit,
    },
    boil_size: {
      value: batchSize * 1.3, // Default 30% boil-off
      unit: batchSizeUnit,
    },
    boil_time: {
      value: 60,
      unit: 'min',
    },
    efficiency: {
      brewhouse: 72,
      conversion: 100,
      lauter: 95,
      mash: 95,
    },
    style: selectedStyle as StyleType,
    ingredients: {
      fermentable_additions: [],
      hop_additions: [],
      culture_additions: [],
    },
    mash: defaultMash ? structuredClone(defaultMash.mashProfile) : undefined,
  };

  const recipeId = crypto.randomUUID();
  const recipeDoc: RecipeDocument = {
    id: recipeId,
    recipe: newRecipe,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };

  await recipesCollection.insert(recipeDoc);

  return recipeId;
}

/**
 * Validate that a recipe has the required BeerJSON structure
 */
function validateRecipeStructure(recipe: RecipeType): void {
  if (!recipe.name) {
    throw new Error('Recipe must have a name');
  }
  if (!recipe.batch_size) {
    throw new Error('Recipe must have a batch_size');
  }
  if (!recipe.ingredients) {
    throw new Error('Recipe must have an ingredients object');
  }
  // Ensure ingredients arrays exist (can be empty)
  if (!Array.isArray(recipe.ingredients.fermentable_additions)) {
    recipe.ingredients.fermentable_additions = [];
  }
  if (!Array.isArray(recipe.ingredients.hop_additions)) {
    recipe.ingredients.hop_additions = [];
  }
  if (!Array.isArray(recipe.ingredients.culture_additions)) {
    recipe.ingredients.culture_additions = [];
  }
}

/**
 * Update an existing recipe with validation
 */
export async function updateRecipe(params: UpdateRecipeParams): Promise<void> {
  const { recipeId, recipe } = params;

  // Validate the recipe structure before updating
  validateRecipeStructure(recipe);

  await recipesCollection.update(recipeId, (draft) => {
    draft.recipe = recipe;
    draft.updatedAt = Date.now();
  });
}

/**
 * Delete a recipe by ID
 */
export async function deleteRecipe(recipeId: string): Promise<void> {
  await recipesCollection.delete(recipeId);
}

/**
 * Get all available beer styles
 */
export function getAvailableStyles(): Array<{ name: string; category: string }> {
  return styles.map((s) => ({ name: s.name, category: s.category }));
}
