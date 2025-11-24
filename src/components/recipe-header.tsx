import { InlineEditable } from '@/components/inline-editable';
import { EditableStyleSelector } from '@/components/editable-style-selector';
import { DeleteRecipeDialog } from '@/components/delete-recipe-dialog';
import { recipesCollection, batchesCollection, equipmentCollection, DEFAULT_EQUIPMENT } from '@/db';
import { getAvailableStyles } from '@/lib/actions/recipes';
import type { RecipeDocument } from '@/db';
import styles from '@/data/styles.json';
import { Button } from '@/components/ui/button';
import { Beaker } from 'lucide-react';
import { useNavigate } from '@tanstack/react-router';
import { useLiveQuery } from '@tanstack/react-db';

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
  const navigate = useNavigate();
  const { data: batches } = useLiveQuery(batchesCollection);
  const { data: equipmentProfiles } = useLiveQuery(equipmentCollection);

  const handleNameUpdate = async (newName: string) => {
    await recipesCollection.update(recipe.id, (draft) => {
      draft.recipe.name = newName;
      draft.updatedAt = Date.now();
    });
  };

  const handleStyleUpdate = async (newStyleName: string) => {
    const selectedStyle = styles.find((s) => s.name === newStyleName);
    if (!selectedStyle) {
      throw new Error(`Style "${newStyleName}" not found`);
    }

    await recipesCollection.update(recipe.id, (draft) => {
      draft.recipe.style = selectedStyle as any;
      draft.updatedAt = Date.now();
    });
  };

  const handleBrew = async () => {
    // Count existing batches for this recipe to generate batch number
    const recipeBatches = batches?.filter(b => b.recipeId === recipe.id) || [];
    const batchNumber = recipeBatches.length + 1;

    // Get default equipment or use first available
    const equipment = equipmentProfiles?.find(e => e.id === "default")
      || equipmentProfiles?.[0]
      || DEFAULT_EQUIPMENT;

    const batch = {
      id: `batch-${Date.now()}`,
      name: `${recipe.recipe.name} Batch ${batchNumber}`,
      recipeId: recipe.id,
      equipmentId: equipment.id,
      recipe: structuredClone(recipe.recipe), // Deep copy of recipe
      equipment: structuredClone(equipment.equipment), // Deep copy of equipment
      brewDate: Date.now(),
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };

    await batchesCollection.insert(batch);
    navigate({ to: `/batches/${batch.id}` });
  };

  return (
    <div className="flex items-start justify-between gap-4">
      <div className="flex-1 min-w-0">
        <InlineEditable
          value={recipe.recipe.name}
          onSave={handleNameUpdate}
          displayAs="heading"
          placeholder="Recipe name"
        />
        <div className="flex items-center gap-3 mt-1 flex-wrap">
          {recipe.recipe.style && (
            <div className="flex items-center gap-3 flex-wrap">
              <EditableStyleSelector
                styleName={recipe.recipe.style.name}
                styleCategory={recipe.recipe.style.category}
                onSave={handleStyleUpdate}
              />
              {recipe.recipe.author && (
                <p className="text-xs text-muted-foreground/70 shrink-0">
                  by {recipe.recipe.author}
                </p>
              )}
            </div>
          )}
        </div>
      </div>
      <div className="shrink-0 flex items-center gap-2">
        {showBrew && (
          <Button onClick={handleBrew} variant="default">
            <Beaker className="h-4 w-4 mr-2" />
            Brew
          </Button>
        )}
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
