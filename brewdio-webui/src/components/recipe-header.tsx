import { InlineEditable } from "@/components/inline-editable";
import { EditableStyleSelector } from "@/components/editable-style-selector";
import { EditableTypeSelector } from "@/components/editable-type-selector";
import { EditableEquipmentSelector } from "@/components/editable-equipment-selector";
import { DeleteRecipeDialog } from "@/components/delete-recipe-dialog";
import { BrewBatchDialog } from "@/components/brew-batch-dialog";
import type { RecipeDocument } from "@/lib/db/recipes";
import { getRecipeDb, recipeKeys } from "@/lib/db/recipes";
import { useQueryClient } from "@tanstack/react-query";
import { getStyles, type EquipmentProfile } from "brewdio-wasm";

interface RecipeHeaderProps {
  recipe: RecipeDocument;
  showDelete?: boolean;
  redirectOnDelete?: boolean;
  showBrew?: boolean;
}

export function RecipeHeader({
  recipe,
  showDelete = true,
  redirectOnDelete = false,
  showBrew = true,
}: RecipeHeaderProps) {
  const queryClient = useQueryClient();

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: recipeKeys.all });
    queryClient.invalidateQueries({ queryKey: recipeKeys.detail(recipe.id) });
  };

  const handleNameUpdate = async (newName: string) => {
    const db = getRecipeDb();
    const updated = structuredClone(recipe.recipe);
    updated.name = newName;
    db.updateRecipe(recipe.id, newName, updated as any);
    invalidate();
  };

  const handleTypeUpdate = async (newType: string) => {
    const db = getRecipeDb();
    const updated = structuredClone(recipe.recipe);
    updated.type = newType as any;
    db.updateRecipe(recipe.id, updated.name, updated as any);
    invalidate();
  };

  const handleStyleUpdate = async (newStyleName: string) => {
    const selectedStyle = getStyles().find((s) => s.name === newStyleName);
    if (!selectedStyle) {
      throw new Error(`Style "${newStyleName}" not found`);
    }

    const db = getRecipeDb();
    const updated = structuredClone(recipe.recipe);
    // Extract only StyleBase fields for RecipeStyleType
    // (StyleType has extra fields that cause autosurgeon reconcile to fail)
    updated.style = {
      name: selectedStyle.name,
      category: selectedStyle.category,
      category_number: selectedStyle.category_number,
      style_guide: selectedStyle.style_guide,
      style_letter: selectedStyle.style_letter,
      type: (selectedStyle as any).type,
    } as any;
    console.log(updated);
    db.updateRecipe(recipe.id, updated.name, updated as any);
    invalidate();
  };

  const handleEquipmentUpdate = async (profile: EquipmentProfile | null) => {
    const db = getRecipeDb();
    if (profile) {
      db.setRecipeEquipment(recipe.id, profile.equipment as any, profile.id);
      // Copy efficiency into the recipe
      const updated = structuredClone(recipe.recipe);
      updated.efficiency = profile.efficiency;
      db.updateRecipe(recipe.id, updated.name, updated as any);
    } else {
      db.setRecipeEquipment(recipe.id, undefined as any, undefined);
    }
    invalidate();
  };

  return (
    <div className="flex items-start justify-between gap-4">
      <div className="flex-1 min-w-0">
        <div className="flex items-baseline gap-3">
          <InlineEditable
            value={recipe.recipe.name}
            onSave={handleNameUpdate}
            displayAs="heading"
            placeholder="Recipe name"
          />
          {recipe.recipe.author && (
            <p className="text-xs text-muted-foreground/70 shrink-0">
              by {recipe.recipe.author}
            </p>
          )}
        </div>
        <div className="flex items-center gap-3 mt-1 flex-wrap">
          {recipe.recipe.style && (
            <div className="flex items-center gap-3 flex-wrap">
              <EditableStyleSelector
                styleName={recipe.recipe.style.name}
                styleCategory={recipe.recipe.style.category}
                onSave={handleStyleUpdate}
              />
              <span className="text-muted-foreground/40">·</span>
              <EditableTypeSelector
                type={recipe.recipe.type}
                onSave={handleTypeUpdate}
              />
            </div>
          )}
          <div className="ml-auto">
            <EditableEquipmentSelector
              equipment={recipe.equipment}
              onSave={handleEquipmentUpdate}
            />
          </div>
        </div>
      </div>
      <div className="shrink-0 flex items-center gap-2">
        {showBrew && <BrewBatchDialog recipe={recipe} />}
        {showDelete && (
          <DeleteRecipeDialog
            recipeId={recipe.id}
            recipeName={recipe.recipe.name}
            redirectOnDelete={redirectOnDelete}
          />
        )}
      </div>
    </div>
  );
}
