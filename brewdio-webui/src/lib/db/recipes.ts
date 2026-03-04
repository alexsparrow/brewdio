import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { RecipeType } from 'brewdio-wasm';
import { getBackend } from './app-db';
import { recipeKeys } from './query-keys';
import type { Recipe } from './backend';

export { recipeKeys };

// Re-export the typed Recipe from backend for consumer compatibility.
export type { Recipe };

/** Alias for backwards compatibility with existing code. */
export type RecipeDocument = Recipe;

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/** Fetch all (non-deleted) recipes. */
export function useRecipes() {
  return useQuery<Recipe[]>({
    queryKey: recipeKeys.all,
    queryFn: () => getBackend().listRecipes(),
  });
}

/** Fetch a single recipe by ID. */
export function useRecipe(id: string) {
  return useQuery<Recipe | null>({
    queryKey: recipeKeys.detail(id),
    queryFn: () => getBackend().getRecipe(id),
  });
}

// ---------------------------------------------------------------------------
// Mutations
// ---------------------------------------------------------------------------

export function useCreateRecipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name, recipe }: { name: string; recipe: RecipeType }) =>
      getBackend().createRecipe(name, recipe),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: recipeKeys.all });
    },
  });
}

export function useUpdateRecipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name, recipe }: { id: string; name: string; recipe: RecipeType }) =>
      getBackend().updateRecipe(id, name, recipe),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: recipeKeys.all });
      queryClient.invalidateQueries({ queryKey: recipeKeys.detail(variables.id) });
    },
  });
}

export function useDeleteRecipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => getBackend().deleteRecipe(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: recipeKeys.all });
    },
  });
}

// ---------------------------------------------------------------------------
// Imperative helpers (for use outside React, e.g. AI tools, actions)
// ---------------------------------------------------------------------------

/** Create a recipe imperatively. Returns the new recipe ID. */
export async function createRecipeImperative(name: string, recipe: RecipeType): Promise<string> {
  return getBackend().createRecipe(name, recipe);
}

/** Update a recipe imperatively. */
export async function updateRecipeImperative(id: string, name: string, recipe: RecipeType): Promise<void> {
  return getBackend().updateRecipe(id, name, recipe);
}

/** Delete a recipe imperatively. */
export async function deleteRecipeImperative(id: string): Promise<void> {
  return getBackend().deleteRecipe(id);
}

/** List all recipes imperatively. */
export async function listRecipesImperative(): Promise<Recipe[]> {
  return getBackend().listRecipes();
}

/** Get a recipe by ID imperatively. */
export async function getRecipeImperative(id: string): Promise<Recipe | null> {
  return getBackend().getRecipe(id);
}
