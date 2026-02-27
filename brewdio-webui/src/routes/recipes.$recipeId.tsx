import { createFileRoute, useRouter } from "@tanstack/react-router";
import { useRecipe, getRecipeDb, recipeKeys } from "@/lib/db/recipes";
import { useQueryClient } from "@tanstack/react-query";
import { RecipeEditProvider } from "@/contexts/recipe-edit-context";
import { AddFermentableDialog } from "@/components/add-fermentable-dialog";
import { AddHopDialog } from "@/components/add-hop-dialog";
import { AddCultureDialog } from "@/components/add-culture-dialog";
import { RecipeHeader } from "@/components/recipe-header";
import { RecipeMashSection } from "@/components/recipe-mash-section";
import { InlineEditableWithUnit } from "@/components/inline-editable";
import { EditableNotes } from "@/components/editable-notes";
import { Button } from "@/components/ui/button";
import { Trash2, Pencil } from "lucide-react";
import { useEffect, useMemo } from "react";
import { useCalculation } from "@/hooks/use-calculation";
import { stores } from "@/lib/calculate";
import { RetroCockpitDial } from "@/components/retro-cockpit-dial";
import { GravitySightGlass } from "@/components/gravity-sight-glass";
import { Screw } from "@/components/screw";
import { srmToSrgb, rgbToHex, colorToSrm, styleForRecipe } from "brewdio-wasm";
import { GrainIcon, HopIcon, YeastIcon } from "@/components/ingredient-icons";
import type { FermentableAdditionType, RecipeType, VolumeUnitType, TimeUnitType } from "brewdio-wasm";

export const Route = createFileRoute("/recipes/$recipeId")({
  component: RecipeDetailComponent,
});

/**
 * Get hex color for a fermentable based on its SRM value
 */
function getFermentableColorHex(fermentable: FermentableAdditionType): string | undefined {
  const colorData = fermentable.color;

  if (!colorData) {
    return undefined;
  }

  try {
    const srm = colorToSrm(colorData);
    const rgb = srmToSrgb(srm) as [number, number, number];
    return rgbToHex(rgb[0], rgb[1], rgb[2]);
  } catch {
    return undefined;
  }
}

function RecipeDetailComponent() {
  const { recipeId } = Route.useParams();
  const { data: recipe, status } = useRecipe(recipeId);
  const router = useRouter();
  const queryClient = useQueryClient();
  const og = useCalculation<number>("og");
  const fg = useCalculation<number>("fg");
  const abv = useCalculation<number>("abv");
  const color = useCalculation<number>("color");
  const ibu = useCalculation<number>("ibu");

  // Extract style ranges if available - must be before any conditional returns
  const styleRanges = useMemo(() => {
    if (!recipe?.recipe) return null;

    const style = styleForRecipe(recipe.recipe);
    if (!style) return null;

    // Helper to convert beerjson range types to simple min/max
    const getRange = (range: any) => {
      if (!range) return null;
      return {
        min: range.minimum?.value ?? null,
        max: range.maximum?.value ?? null
      };
    };

    return {
      og: getRange(style.original_gravity),
      fg: getRange(style.final_gravity),
      ibu: getRange(style.international_bitterness_units),
      color: getRange(style.color), // This is in SRM
      abv: getRange(style.alcohol_by_volume)
    };
  }, [recipe?.recipe]);

  const updateRecipe = (mutateFn: (r: RecipeType) => void) => {
    if (!recipe) return;
    const updated = structuredClone(recipe.recipe);
    mutateFn(updated);
    const db = getRecipeDb();
    db.updateRecipe(recipeId, updated.name, updated);
    queryClient.invalidateQueries({ queryKey: recipeKeys.all });
    queryClient.invalidateQueries({ queryKey: recipeKeys.detail(recipeId) });
  };

  const handleRemoveFermentable = (index: number) => {
    updateRecipe((r) => {
      r.ingredients.fermentable_additions =
        r.ingredients.fermentable_additions?.filter((_, idx) => idx !== index) || [];
    });
  };

  const handleRemoveHop = (index: number) => {
    updateRecipe((r) => {
      r.ingredients.hop_additions =
        r.ingredients.hop_additions?.filter((_, idx) => idx !== index) || [];
    });
  };

  const handleRemoveCulture = (index: number) => {
    updateRecipe((r) => {
      r.ingredients.culture_additions =
        r.ingredients.culture_additions?.filter((_, idx) => idx !== index) || [];
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

  // Sync recipe data to calculation store
  // This triggers all calculations to update when the recipe changes
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

  const { recipe: beerRecipe } = recipe;

  const handleBatchSizeUpdate = (newValue: number, newUnit: string) => {
    updateRecipe((r) => { r.batch_size = { value: newValue, unit: newUnit as VolumeUnitType }; });
  };

  const handleBoilSizeUpdate = (newValue: number, newUnit: string) => {
    updateRecipe((r) => {
      if (!r.boil) r.boil = { boil_time: { value: 60, unit: "min" } };
      r.boil.pre_boil_size = { value: newValue, unit: newUnit as VolumeUnitType };
    });
  };

  const handleBoilTimeUpdate = (newValue: number, newUnit: string) => {
    updateRecipe((r) => {
      if (!r.boil) r.boil = { boil_time: { value: 60, unit: "min" } };
      r.boil.boil_time = { value: newValue, unit: newUnit as TimeUnitType };
    });
  };

  const handleEfficiencyUpdate = (newValue: number, _unit: string) => {
    updateRecipe((r) => {
      if (!r.efficiency) {
        r.efficiency = {
          brewhouse: { value: newValue, unit: "%" },
          conversion: { value: 100, unit: "%" },
          lauter: { value: 95, unit: "%" },
          mash: { value: 95, unit: "%" },
        };
      } else {
        r.efficiency.brewhouse = { value: newValue, unit: "%" };
      }
    });
  };

  const handleNotesUpdate = (newNotes: string) => {
    updateRecipe((r) => { r.notes = newNotes; });
  };

  const liquidColor = color ? (() => { const rgb = srmToSrgb(color) as [number, number, number]; return rgbToHex(rgb[0], rgb[1], rgb[2]); })() : "#FBB123";

  return (
    <RecipeEditProvider
      id={recipeId}
      document={recipe}
      type="recipe"
    >
      <div className="flex flex-col gap-6">
        <RecipeHeader recipe={recipe} redirectOnDelete={true} />

      {/* Calculated Values Section - Retro Cockpit Panel */}
      <div className="relative bg-gray-100 dark:bg-gray-900 border-t-2 border-gray-300 dark:border-gray-700 rounded-xl p-4 shadow-xl overflow-hidden">
        {/* Subtle texture overlay */}
        <div className="absolute inset-0 opacity-5 pointer-events-none bg-[url('https://www.transparenttextures.com/patterns/brushed-alum-dark.png')]"></div>

        {/* Panel Mounting Screws */}
        <Screw tl />
        <Screw tr />
        <Screw bl />
        <Screw br />

        {/* Main Grid Layout */}
        <div className="grid grid-cols-1 md:grid-cols-12 gap-4 relative z-10 p-2">
          {/* LEFT COLUMN: GRAVITY SECTION */}
          <div className="md:col-span-3 flex justify-around bg-white/50 dark:bg-gray-950/30 p-3 rounded-lg border border-gray-300 dark:border-gray-800">
            <GravitySightGlass
              label="OG"
              value={og ?? 1.0}
              liquidColor={liquidColor}
              percentage={0} // Calculated internally now
              min={1.0}
              max={1.1}
              targetMin={styleRanges?.og?.min ?? 1.04}
              targetMax={styleRanges?.og?.max ?? 1.07}
            />
            {/* Arrow indicating change */}
            <div className="self-center text-gray-600 dark:text-gray-500 text-2xl">
              →
            </div>
            <GravitySightGlass
              label="FG"
              value={fg ?? 1.0}
              liquidColor={liquidColor}
              percentage={0} // Calculated internally now
              min={1.0}
              max={1.04}
              targetMin={styleRanges?.fg?.min ?? 1.008}
              targetMax={styleRanges?.fg?.max ?? 1.016}
            />
          </div>

          {/* RIGHT COLUMN: DIALS SECTION */}
          <div className="md:col-span-9 grid grid-cols-1 sm:grid-cols-3 gap-3">
            {/* IBU DIAL - With target range highlighted */}
            <div className="bg-white/50 dark:bg-gray-950/30 p-2 rounded-lg border border-gray-300 dark:border-gray-800 flex items-center justify-center">
              <RetroCockpitDial
                label="IBU"
                value={ibu ?? 0}
                min={0}
                max={100}
                targetMin={styleRanges?.ibu?.min ?? 20}
                targetMax={styleRanges?.ibu?.max ?? 60}
              />
            </div>

            {/* COLOUR DIAL - With SRM gradient in target range */}
            <div className="bg-white/50 dark:bg-gray-950/30 p-2 rounded-lg border border-gray-300 dark:border-gray-800 flex items-center justify-center">
              <RetroCockpitDial
                label="SRM"
                value={color !== undefined ? color : 0} // Convert EBC to SRM
                min={0}
                max={40}
                targetMin={styleRanges?.color ? styleRanges.color.min : 4}
                targetMax={styleRanges?.color ? styleRanges.color.max : 20}
                showSrmGradient={true}
              />
            </div>

            {/* ABV DIAL */}
            <div className="bg-white/50 dark:bg-gray-950/30 p-2 rounded-lg border border-gray-300 dark:border-gray-800 flex items-center justify-center">
              <RetroCockpitDial
                label="ABV"
                value={abv ?? 0}
                min={0}
                max={12}
                targetMin={styleRanges?.abv?.min ?? 4}
                targetMax={styleRanges?.abv?.max ?? 7}
              />
            </div>
          </div>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="border rounded-lg p-4">
          <h2 className="text-xl font-semibold mb-3">Recipe Details</h2>
          <div className="space-y-2">
            <InlineEditableWithUnit
              value={beerRecipe.batch_size.value}
              unit={beerRecipe.batch_size.unit}
              availableUnits={["gal", "l", "ml"]}
              onSave={handleBatchSizeUpdate}
              label="Batch Size"
            />
            {beerRecipe.boil?.pre_boil_size && (
              <InlineEditableWithUnit
                value={beerRecipe.boil.pre_boil_size.value}
                unit={beerRecipe.boil.pre_boil_size.unit}
                availableUnits={["gal", "l", "ml"]}
                onSave={handleBoilSizeUpdate}
                label="Boil Size"
              />
            )}
            {beerRecipe.boil?.boil_time && (
              <InlineEditableWithUnit
                value={beerRecipe.boil.boil_time.value}
                unit={beerRecipe.boil.boil_time.unit}
                availableUnits={["min", "hr"]}
                onSave={handleBoilTimeUpdate}
                label="Boil Time"
              />
            )}
            {beerRecipe.efficiency && (
              <InlineEditableWithUnit
                value={beerRecipe.efficiency.brewhouse.value}
                unit="%"
                availableUnits={["%"]}
                onSave={handleEfficiencyUpdate}
                label="Brewhouse Efficiency"
              />
            )}
          </div>
        </div>

        <div className="border rounded-lg p-4">
          <h2 className="text-xl font-semibold mb-3">Notes</h2>
          <EditableNotes
            value={beerRecipe.notes || ""}
            onSave={handleNotesUpdate}
            placeholder="Add brewing notes, recipe history, tasting notes, etc..."
          />
        </div>
      </div>

      <div id="fermentables" className="border rounded-lg p-4 scroll-mt-20">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xl font-semibold">Fermentables</h2>
          <AddFermentableDialog />
        </div>
        {beerRecipe.ingredients.fermentable_additions?.length === 0 ? (
          <div className="text-muted-foreground text-sm">
            No fermentables added yet
          </div>
        ) : (
          <div className="space-y-2">
            {beerRecipe.ingredients.fermentable_additions?.map((ferm, idx) => (
              <div
                key={idx}
                className="flex justify-between items-center text-sm border-b pb-2 last:border-0"
              >
                <div className="flex items-center gap-1.5">
                  <GrainIcon
                    className="h-6 w-6 flex-shrink-0"
                    srmColor={getFermentableColorHex(ferm)}
                  />
                  <span className="font-medium">
                    {ferm.name || "Unnamed fermentable"}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground">
                    {ferm.amount.value} {ferm.amount.unit}
                  </span>
                  <AddFermentableDialog
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
          <AddHopDialog />
        </div>
        {beerRecipe.ingredients.hop_additions?.length === 0 ? (
          <div className="text-muted-foreground text-sm">No hops added yet</div>
        ) : (
          <div className="space-y-2">
            {beerRecipe.ingredients.hop_additions?.map((hop, idx) => (
              <div
                key={idx}
                className="flex justify-between items-center text-sm border-b pb-2 last:border-0"
              >
                <div className="flex items-center gap-1.5">
                  <HopIcon className="h-6 w-6 flex-shrink-0" />
                  <div>
                    <span className="font-medium">
                      {hop.name || "Unnamed hop"}
                    </span>
                    {hop.timing && (
                      <span className="text-muted-foreground ml-2">
                        {hop.timing.use === "add_to_boil" && "Boil"}
                        {hop.timing.use === "add_to_fermentation" && "Dry Hop"}
                        {hop.timing.time && (
                          <> @ {hop.timing.time.value} {hop.timing.time.unit}</>
                        )}
                        {hop.timing.duration && (
                          <> ({hop.timing.duration.value} {hop.timing.duration.unit})</>
                        )}
                      </span>
                    )}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground">
                    {hop.amount.value} {hop.amount.unit}
                  </span>
                  <AddHopDialog
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
          <AddCultureDialog />
        </div>
        {beerRecipe.ingredients.culture_additions?.length === 0 ? (
          <div className="text-muted-foreground text-sm">
            No cultures added yet
          </div>
        ) : (
          <div className="space-y-2">
            {beerRecipe.ingredients.culture_additions?.map((culture, idx) => (
              <div
                key={idx}
                className="flex justify-between items-center text-sm border-b pb-2 last:border-0"
              >
                <div className="flex items-center gap-1.5">
                  <YeastIcon className="h-6 w-6 flex-shrink-0" />
                  <span className="font-medium">
                    {culture.name || "Unnamed culture"}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  {culture.amount && (
                    <span className="text-muted-foreground">
                      {culture.amount.value} {culture.amount.unit}
                    </span>
                  )}
                  <AddCultureDialog
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

      <div id="mash" className="border rounded-lg p-4 scroll-mt-20">
        <RecipeMashSection />
      </div>
    </div>
    </RecipeEditProvider>
  );
}
