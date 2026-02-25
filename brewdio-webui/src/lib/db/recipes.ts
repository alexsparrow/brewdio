import { useQuery, useMutation, useQueryClient, type QueryClient } from '@tanstack/react-query';
import type { RecipeType, EquipmentType } from 'brewdio-wasm';
import type { RecipeDb as RecipeDbClass } from 'brewdio-wasm';

// Re-export the recipe document type used throughout the app.
// This replaces the old Dexie-based RecipeDocument from db.ts.
export interface RecipeDocument {
  id: string;
  name: string;
  recipe: RecipeType;
  equipment?: EquipmentType;
}

// ---------------------------------------------------------------------------
// Singleton RecipeDb instance
// ---------------------------------------------------------------------------

let recipeDb: RecipeDbClass | null = null;

/** Call once during app initialisation (before React renders). */
export function initRecipeDb(db: RecipeDbClass) {
  recipeDb = db;
}

export function getRecipeDb(): RecipeDbClass {
  if (!recipeDb) throw new Error('RecipeDb not initialised – call initRecipeDb() first');
  return recipeDb;
}

/**
 * Wire up the WASM change-notification callbacks so that external mutations
 * (e.g. incoming sync) automatically invalidate the TanStack Query cache.
 */
export function registerChangeCallback(queryClient: QueryClient) {
  const db = getRecipeDb();
  db.onRecipesChange(() => {
    queryClient.invalidateQueries({ queryKey: ['recipes'] });
  });
  db.onBatchesChange(() => {
    queryClient.invalidateQueries({ queryKey: ['batches'] });
  });
  db.onSettingsChange(() => {
    queryClient.invalidateQueries({ queryKey: ['settings'] });
  });
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
      const db = getRecipeDb();
      return db.listRecipes() as unknown as RecipeDocument[];
    },
  });
}

/** Fetch a single recipe by ID. */
export function useRecipe(id: string) {
  return useQuery<RecipeDocument | null>({
    queryKey: recipeKeys.detail(id),
    queryFn: () => {
      const db = getRecipeDb();
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
      const db = getRecipeDb();
      const id = db.createRecipe(name, recipe as any);
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
      const db = getRecipeDb();
      db.updateRecipe(id, name, recipe as any);
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
      const db = getRecipeDb();
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
  const db = getRecipeDb();
  return db.createRecipe(name, recipe as any);
}

/** Update a recipe imperatively. */
export function updateRecipeImperative(id: string, name: string, recipe: RecipeType): void {
  const db = getRecipeDb();
  db.updateRecipe(id, name, recipe as any);
}

/** Delete a recipe imperatively. */
export function deleteRecipeImperative(id: string): void {
  const db = getRecipeDb();
  db.deleteRecipe(id);
}

/** List all recipes imperatively. */
export function listRecipesImperative(): RecipeDocument[] {
  const db = getRecipeDb();
  return db.listRecipes() as unknown as RecipeDocument[];
}

/** Get a recipe by ID imperatively. */
export function getRecipeImperative(id: string): RecipeDocument | null {
  const db = getRecipeDb();
  const result = db.getRecipe(id);
  return (result ?? null) as unknown as RecipeDocument | null;
}
