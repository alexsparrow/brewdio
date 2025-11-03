import { createFileRoute, Link } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { recipesCollection } from "@/db";
import { CreateRecipeDialog } from "@/components/create-recipe-dialog";
import { DeleteRecipeDialog } from "@/components/delete-recipe-dialog";

export const Route = createFileRoute("/")({
  component: HomeComponent,
});

function HomeComponent() {
  const { data: recipes, status } = useLiveQuery(recipesCollection);

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Recipes</h1>
        <CreateRecipeDialog />
      </div>

      {status === "loading" && <div>Loading recipes...</div>}

      {status === "ready" && recipes.length === 0 && (
        <div className="text-muted-foreground text-center py-12">
          No recipes yet. Create your first recipe to get started!
        </div>
      )}

      {status === "ready" && recipes.length > 0 && (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {recipes.map((recipeDoc) => (
            <div key={recipeDoc.id} className="border rounded-lg p-4 hover:bg-accent transition-colors">
              <Link
                to="/recipes/$recipeId"
                params={{ recipeId: recipeDoc.id }}
                className="block"
              >
                <h2 className="text-xl font-semibold mb-2">
                  {recipeDoc.recipe.name}
                </h2>
                <div className="text-sm text-muted-foreground space-y-1">
                  <div>Type: {recipeDoc.recipe.type}</div>
                  <div>
                    Batch Size: {recipeDoc.recipe.batch_size.value}{" "}
                    {recipeDoc.recipe.batch_size.unit}
                  </div>
                  {recipeDoc.recipe.style && (
                    <div>Style: {recipeDoc.recipe.style.name}</div>
                  )}
                </div>
              </Link>
              <div className="mt-3 pt-3 border-t" onClick={(e) => e.stopPropagation()}>
                <DeleteRecipeDialog
                  recipeId={recipeDoc.id}
                  recipeName={recipeDoc.recipe.name}
                  variant="outline"
                  size="sm"
                />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
