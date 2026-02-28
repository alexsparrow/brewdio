import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { RecipeType, EquipmentType } from 'brewdio-wasm';
import { getAppDb } from './app-db';

// Re-export the recipe document type used throughout the app.
export interface RecipeDocument {
  id: string;
  name: string;
  recipe: RecipeType;
  equipment?: EquipmentType;
}

// ---------------------------------------------------------------------------
// Query keys
// ---------------------------------------------------------------------------

export const recipeKeys = {
  all: ['recipes'] as const,
  detail: (id: string) => ['recipes', id] as const,
};

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/** Fetch all (non-deleted) recipes. */
export function useRecipes() {
  return useQuery<RecipeDocument[]>({
    queryKey: recipeKeys.all,
    queryFn: () => {
      const db = getAppDb();
      return db.listRecipes() as unknown as RecipeDocument[];
    },
  });
}

/** Fetch a single recipe by ID. */
export function useRecipe(id: string) {
  return useQuery<RecipeDocument | null>({
    queryKey: recipeKeys.detail(id),
    queryFn: () => {
      const db = getAppDb();
      const result = db.getRecipe(id);
      return (result ?? null) as unknown as RecipeDocument | null;
    },
  });
}

// ---------------------------------------------------------------------------
// Mutations
// ---------------------------------------------------------------------------

export function useCreateRecipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name, recipe }: { name: string; recipe: RecipeType }) => {
      const db = getAppDb();
      const id = db.createRecipe(name, recipe);
      return Promise.resolve(id);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: recipeKeys.all });
    },
  });
}

export function useUpdateRecipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name, recipe }: { id: string; name: string; recipe: RecipeType }) => {
      const db = getAppDb();
      db.updateRecipe(id, name, recipe);
      return Promise.resolve();
    },
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: recipeKeys.all });
      queryClient.invalidateQueries({ queryKey: recipeKeys.detail(variables.id) });
    },
  });
}

export function useDeleteRecipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => {
      const db = getAppDb();
      db.deleteRecipe(id);
      return Promise.resolve();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: recipeKeys.all });
    },
  });
}

// ---------------------------------------------------------------------------
// Imperative helpers (for use outside React, e.g. AI tools, actions)
// ---------------------------------------------------------------------------

/** Create a recipe imperatively. Returns the new recipe ID. */
export function createRecipeImperative(name: string, recipe: RecipeType): string {
  const db = getAppDb();
  return db.createRecipe(name, recipe);
}

/** Update a recipe imperatively. */
export function updateRecipeImperative(id: string, name: string, recipe: RecipeType): void {
  const db = getAppDb();
  db.updateRecipe(id, name, recipe);
}

/** Delete a recipe imperatively. */
export function deleteRecipeImperative(id: string): void {
  const db = getAppDb();
  db.deleteRecipe(id);
}

/** List all recipes imperatively. */
export function listRecipesImperative(): RecipeDocument[] {
  const db = getAppDb();
  return db.listRecipes() as unknown as RecipeDocument[];
}

/** Get a recipe by ID imperatively. */
export function getRecipeImperative(id: string): RecipeDocument | null {
  const db = getAppDb();
  const result = db.getRecipe(id);
  return (result ?? null) as unknown as RecipeDocument | null;
}
