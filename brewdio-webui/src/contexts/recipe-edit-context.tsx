import { createContext, useContext, useCallback, type ReactNode } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { RecipeDocument } from "@/lib/db/recipes";
import { getRecipeDb, recipeKeys } from "@/lib/db/recipes";
import { updateBatchImperative, batchKeys, type BatchDocument } from "@/lib/db/batches";
import type { RecipeType } from "brewdio-wasm";

/**
 * Context for managing the target of recipe editing.
 * This allows components to work with either recipes or batches
 * without needing to pass collection props down the tree.
 */
interface RecipeEditContextValue {
  /** The ID of the recipe or batch being edited */
  id: string;
  /** The recipe or batch document */
  document: RecipeDocument | BatchDocument;
  /** Update function — accepts a draft callback that receives a mutable copy of the document */
  update: (draftFn: (draft: { recipe: RecipeType; updatedAt?: number }) => void) => void;
  /** Type of document being edited */
  type: "recipe" | "batch";
}

const RecipeEditContext = createContext<RecipeEditContextValue | null>(null);

interface RecipeEditProviderProps {
  id: string;
  document: RecipeDocument | BatchDocument;
  type: "recipe" | "batch";
  children: ReactNode;
}

export function RecipeEditProvider({
  id,
  document,
  type,
  children,
}: RecipeEditProviderProps) {
  const queryClient = useQueryClient();

  const update = useCallback(
    (draftFn: (draft: { recipe: RecipeType; updatedAt?: number }) => void) => {
      if (type === "recipe") {
        // Use WASM db for recipes
        const db = getRecipeDb();
        // Get the current recipe from the db to ensure we have the latest
        const current = db.getRecipe(id) as unknown as RecipeDocument | null;
        if (!current) return;
        // Apply the draft function to a mutable copy
        const draft = { recipe: structuredClone(current.recipe) } as { recipe: RecipeType; updatedAt?: number };
        draftFn(draft);
        db.updateRecipe(id, draft.recipe.name, draft.recipe);
        // Invalidate queries so UI updates
        queryClient.invalidateQueries({ queryKey: recipeKeys.all });
        queryClient.invalidateQueries({ queryKey: recipeKeys.detail(id) });
      } else {
        // Use WASM db for batches
        const batchDoc = document as BatchDocument;
        const draft = {
          recipe: structuredClone(batchDoc.recipe),
          updatedAt: batchDoc.updatedAt,
        };
        draftFn(draft);
        updateBatchImperative(id, batchDoc.name, {
          ...batchDoc,
          recipe: draft.recipe,
          updatedAt: draft.updatedAt ?? Date.now(),
        });
        queryClient.invalidateQueries({ queryKey: batchKeys.all });
        queryClient.invalidateQueries({ queryKey: batchKeys.detail(id) });
      }
    },
    [id, type, document, queryClient]
  );

  return (
    <RecipeEditContext.Provider value={{ id, document, update, type }}>
      {children}
    </RecipeEditContext.Provider>
  );
}

/**
 * Hook to access the recipe editing context.
 * Must be used within a RecipeEditProvider.
 */
export function useRecipeEdit() {
  const context = useContext(RecipeEditContext);
  if (!context) {
    throw new Error("useRecipeEdit must be used within a RecipeEditProvider");
  }
  return context;
}
