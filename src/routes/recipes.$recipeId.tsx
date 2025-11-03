import { createFileRoute, useRouter, Link } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { recipesCollection } from "@/db";
import { AddFermentableDialog } from "@/components/add-fermentable-dialog";
import { AddHopDialog } from "@/components/add-hop-dialog";
import { AddCultureDialog } from "@/components/add-culture-dialog";
import { RecipeHeader } from "@/components/recipe-header";
import { InlineEditableWithUnit } from "@/components/inline-editable";
import { EditableNotes } from "@/components/editable-notes";
import { Button } from "@/components/ui/button";
import { Trash2, Pencil } from "lucide-react";
import { useEffect } from "react";

export const Route = createFileRoute("/recipes/$recipeId")({
  component: RecipeDetailComponent,
});

function RecipeDetailComponent() {
  const { recipeId } = Route.useParams();
  const { data: recipes, status } = useLiveQuery(recipesCollection);
  const router = useRouter();

  const recipe = recipes?.find((r) => r.id === recipeId);

  const handleRemoveFermentable = async (index: number) => {
    await recipesCollection.update(recipeId, (draft) => {
      draft.recipe.ingredients.fermentable_additions =
        draft.recipe.ingredients.fermentable_additions?.filter((_, idx) => idx !== index) || [];
      draft.updatedAt = Date.now();
    });
  };

  const handleRemoveHop = async (index: number) => {
    await recipesCollection.update(recipeId, (draft) => {
      draft.recipe.ingredients.hop_additions =
        draft.recipe.ingredients.hop_additions?.filter((_, idx) => idx !== index) || [];
      draft.updatedAt = Date.now();
    });
  };

  const handleRemoveCulture = async (index: number) => {
    await recipesCollection.update(recipeId, (draft) => {
      draft.recipe.ingredients.culture_additions =
        draft.recipe.ingredients.culture_additions?.filter((_, idx) => idx !== index) || [];
      draft.updatedAt = Date.now();
    });
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

  if (status === "loading") {
    return <div>Loading recipe...</div>;
  }

  if (!recipe) {
    return <div>Recipe not found</div>;
  }

  const { recipe: beerRecipe } = recipe;

  const handleBatchSizeUpdate = async (newValue: number, newUnit: string) => {
    await recipesCollection.update(recipeId, (draft) => {
      draft.recipe.batch_size = { value: newValue, unit: newUnit };
      draft.updatedAt = Date.now();
    });
  };

  const handleBoilSizeUpdate = async (newValue: number, newUnit: string) => {
    await recipesCollection.update(recipeId, (draft) => {
      draft.recipe.boil_size = { value: newValue, unit: newUnit };
      draft.updatedAt = Date.now();
    });
  };

  const handleBoilTimeUpdate = async (newValue: number, newUnit: string) => {
    await recipesCollection.update(recipeId, (draft) => {
      draft.recipe.boil_time = { value: newValue, unit: newUnit };
      draft.updatedAt = Date.now();
    });
  };

  const handleEfficiencyUpdate = async (newValue: number, unit: string) => {
    // Unit is always '%' for efficiency, but we accept it for consistency
    await recipesCollection.update(recipeId, (draft) => {
      if (!draft.recipe.efficiency) {
        draft.recipe.efficiency = {
          brewhouse: newValue,
          conversion: 100,
          lauter: 95,
          mash: 95,
        };
      } else {
        draft.recipe.efficiency.brewhouse = newValue;
      }
      draft.updatedAt = Date.now();
    });
  };

  const handleNotesUpdate = async (newNotes: string) => {
    await recipesCollection.update(recipeId, (draft) => {
      draft.recipe.notes = newNotes;
      draft.updatedAt = Date.now();
    });
  };

  return (
    <div className="flex flex-col gap-6">
      <RecipeHeader recipe={recipe} redirectOnDelete={true} />

      <div className="grid gap-4 md:grid-cols-2">
        <div className="border rounded-lg p-4">
          <h2 className="text-xl font-semibold mb-3">Recipe Details</h2>
          <div className="space-y-2">
            <div className="text-sm py-1">
              <span className="font-medium">Type:</span> {beerRecipe.type}
            </div>
            <InlineEditableWithUnit
              value={beerRecipe.batch_size.value}
              unit={beerRecipe.batch_size.unit}
              availableUnits={['gal', 'l', 'ml']}
              onSave={handleBatchSizeUpdate}
              label="Batch Size"
            />
            <InlineEditableWithUnit
              value={beerRecipe.boil_size.value}
              unit={beerRecipe.boil_size.unit}
              availableUnits={['gal', 'l', 'ml']}
              onSave={handleBoilSizeUpdate}
              label="Boil Size"
            />
            <InlineEditableWithUnit
              value={beerRecipe.boil_time.value}
              unit={beerRecipe.boil_time.unit}
              availableUnits={['min', 'hr']}
              onSave={handleBoilTimeUpdate}
              label="Boil Time"
            />
            {beerRecipe.efficiency && (
              <InlineEditableWithUnit
                value={beerRecipe.efficiency.brewhouse}
                unit="%"
                availableUnits={['%']}
                onSave={handleEfficiencyUpdate}
                label="Brewhouse Efficiency"
              />
            )}
          </div>
        </div>

        <div className="border rounded-lg p-4">
          <h2 className="text-xl font-semibold mb-3">Notes</h2>
          <EditableNotes
            value={beerRecipe.notes || ''}
            onSave={handleNotesUpdate}
            placeholder="Add brewing notes, recipe history, tasting notes, etc..."
          />
        </div>
      </div>

      <div id="fermentables" className="border rounded-lg p-4 scroll-mt-20">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xl font-semibold">Fermentables</h2>
          <AddFermentableDialog recipeId={recipeId} recipe={recipe} />
        </div>
        {beerRecipe.ingredients.fermentable_additions?.length === 0 ? (
          <div className="text-muted-foreground text-sm">
            No fermentables added yet
          </div>
        ) : (
          <div className="space-y-2">
            {beerRecipe.ingredients.fermentable_additions?.map((ferm, idx) => (
              <div key={idx} className="flex justify-between items-center text-sm border-b pb-2 last:border-0">
                <span className="font-medium">{ferm.name || "Unnamed fermentable"}</span>
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground">
                    {ferm.amount.value} {ferm.amount.unit}
                  </span>
                  <AddFermentableDialog
                    recipeId={recipeId}
                    recipe={recipe}
                    existingFermentable={ferm}
                    index={idx}
                    trigger={
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-8 w-8 p-0 text-muted-foreground hover:text-primary"
                      >
                        <Pencil className="h-4 w-4" />
                      </Button>
                    }
                  />
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
                    onClick={() => handleRemoveFermentable(idx)}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div id="hops" className="border rounded-lg p-4 scroll-mt-20">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xl font-semibold">Hops</h2>
          <AddHopDialog recipeId={recipeId} recipe={recipe} />
        </div>
        {beerRecipe.ingredients.hop_additions?.length === 0 ? (
          <div className="text-muted-foreground text-sm">No hops added yet</div>
        ) : (
          <div className="space-y-2">
            {beerRecipe.ingredients.hop_additions?.map((hop, idx) => (
              <div key={idx} className="flex justify-between items-center text-sm border-b pb-2 last:border-0">
                <div>
                  <span className="font-medium">{hop.name || "Unnamed hop"}</span>
                  {hop.timing?.time && (
                    <span className="text-muted-foreground ml-2">
                      @ {hop.timing.time.value} {hop.timing.time.unit}
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground">
                    {hop.amount.value} {hop.amount.unit}
                  </span>
                  <AddHopDialog
                    recipeId={recipeId}
                    recipe={recipe}
                    existingHop={hop}
                    index={idx}
                    trigger={
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-8 w-8 p-0 text-muted-foreground hover:text-primary"
                      >
                        <Pencil className="h-4 w-4" />
                      </Button>
                    }
                  />
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
                    onClick={() => handleRemoveHop(idx)}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div id="cultures" className="border rounded-lg p-4 scroll-mt-20">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xl font-semibold">Cultures (Yeast)</h2>
          <AddCultureDialog recipeId={recipeId} recipe={recipe} />
        </div>
        {beerRecipe.ingredients.culture_additions?.length === 0 ? (
          <div className="text-muted-foreground text-sm">
            No cultures added yet
          </div>
        ) : (
          <div className="space-y-2">
            {beerRecipe.ingredients.culture_additions?.map((culture, idx) => (
              <div key={idx} className="flex justify-between items-center text-sm border-b pb-2 last:border-0">
                <span className="font-medium">{culture.name || "Unnamed culture"}</span>
                <div className="flex items-center gap-2">
                  {culture.amount && (
                    <span className="text-muted-foreground">
                      {culture.amount.value} {culture.amount.unit}
                    </span>
                  )}
                  <AddCultureDialog
                    recipeId={recipeId}
                    recipe={recipe}
                    existingCulture={culture}
                    index={idx}
                    trigger={
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-8 w-8 p-0 text-muted-foreground hover:text-primary"
                      >
                        <Pencil className="h-4 w-4" />
                      </Button>
                    }
                  />
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
                    onClick={() => handleRemoveCulture(idx)}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
