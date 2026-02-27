import { createFileRoute } from "@tanstack/react-router";
import { useRecipe } from "@/lib/db/recipes";
import { RecipeEditProvider } from "@/contexts/recipe-edit-context";
import { RecipeMashSection } from "@/components/recipe-mash-section";
import { useEffect } from "react";
import { stores } from "@/lib/calculate";

export const Route = createFileRoute("/recipes/$recipeId_/mash")({
  component: RecipeMashComponent,
});

function RecipeMashComponent() {
  const { recipeId } = Route.useParams();
  const { data: recipe, status } = useRecipe(recipeId);

  // Sync recipe data to calculation store
  useEffect(() => {
    if (recipe) {
      stores.state.setState(() => ({
        recipe: recipe.recipe,
      }));
    }
  }, [recipe]);

  if (status === "pending") {
    return <div>Loading recipe...</div>;
  }

  if (!recipe) {
    return <div>Recipe not found</div>;
  }

  return (
    <RecipeEditProvider id={recipeId} document={recipe} type="recipe">
      <div className="flex flex-col gap-6">
        <RecipeMashSection />
      </div>
    </RecipeEditProvider>
  );
}
