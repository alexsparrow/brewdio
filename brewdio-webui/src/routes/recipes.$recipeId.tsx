import { createFileRoute, useNavigate, useRouter } from "@tanstack/react-router";
import { useRecipe, recipeKeys } from "@/lib/db/recipes";
import { getAppDb } from "@/lib/db/app-db";
import { useQueryClient } from "@tanstack/react-query";
import { RecipeEditProvider } from "@/contexts/recipe-edit-context";
import { RecipeHeader } from "@/components/recipe-header";
import { RecipeDetailView } from "@/components/recipe-detail-view";
import { BrewBatchDialog } from "@/components/brew-batch-dialog";
import { useCallback, useEffect, useState } from "react";
import { stores } from "@/lib/calculate";
import { useCommandHandler } from "@/hooks/use-command";
import type { RecipeType } from "brewdio-wasm";

export const Route = createFileRoute("/recipes/$recipeId")({
  component: RecipeDetailComponent,
});

function RecipeDetailComponent() {
  const { recipeId } = Route.useParams();
  const { data: recipe, status } = useRecipe(recipeId);
  const router = useRouter();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [brewDialogOpen, setBrewDialogOpen] = useState(false);

  const updateRecipe = (mutateFn: (r: RecipeType) => void) => {
    if (!recipe) return;
    const updated = structuredClone(recipe.recipe);
    mutateFn(updated);
    const db = getAppDb();
    db.updateRecipe(recipeId, updated.name, updated);
    queryClient.invalidateQueries({ queryKey: recipeKeys.all });
    queryClient.invalidateQueries({ queryKey: recipeKeys.detail(recipeId) });
  };

  // Handle hash navigation for scrolling to sections
  useEffect(() => {
    const hash = router.state.location.hash;
    if (hash) {
      setTimeout(() => {
        const element = document.querySelector(hash);
        if (element) {
          element.scrollIntoView({ behavior: "smooth", block: "start" });
        }
      }, 100);
    }
  }, [router.state.location.hash]);

  // Sync recipe data to calculation store
  useEffect(() => {
    if (recipe) {
      stores.state.setState(() => ({
        recipe: recipe.recipe,
      }));
    }
  }, [recipe]);

  // Register recipe-detail command handlers
  useCommandHandler(
    "recipe.brew",
    useCallback(() => setBrewDialogOpen(true), [])
  );
  useCommandHandler(
    "recipe.jsonEditor",
    useCallback(
      () => navigate({ to: "/recipes/$recipeId/json", params: { recipeId } }),
      [navigate, recipeId]
    )
  );

  if (status === "pending") {
    return <div>Loading recipe...</div>;
  }

  if (!recipe) {
    return <div>Recipe not found</div>;
  }

  return (
    <RecipeEditProvider id={recipeId} document={recipe} type="recipe">
      <BrewBatchDialog recipe={recipe} open={brewDialogOpen} onOpenChange={setBrewDialogOpen} />
      <RecipeDetailView
        recipe={recipe.recipe}
        equipment={recipe.equipment}
        updateRecipe={updateRecipe}
        headerSlot={<RecipeHeader recipe={recipe} redirectOnDelete={true} />}
      />
    </RecipeEditProvider>
  );
}
