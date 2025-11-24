import { createContext, useContext, type ReactNode } from "react";
import type { RecipeDocument } from "@/db";

/**
 * Context for managing the target of recipe editing.
 * This allows components to work with either recipes or batches
 * without needing to pass collection props down the tree.
 */
interface RecipeEditContextValue {
  /** The ID of the recipe or batch being edited */
  id: string;
  /** The recipe or batch document */
  document: RecipeDocument | any;
  /** The collection to use for updates (recipesCollection or batchesCollection) */
  collection: any;
  /** Type of document being edited */
  type: "recipe" | "batch";
}

const RecipeEditContext = createContext<RecipeEditContextValue | null>(null);

interface RecipeEditProviderProps {
  id: string;
  document: RecipeDocument | any;
  collection: any;
  type: "recipe" | "batch";
  children: ReactNode;
}

export function RecipeEditProvider({
  id,
  document,
  collection,
  type,
  children,
}: RecipeEditProviderProps) {
  return (
    <RecipeEditContext.Provider value={{ id, document, collection, type }}>
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
