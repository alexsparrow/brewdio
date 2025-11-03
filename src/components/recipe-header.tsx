import { InlineEditable } from '@/components/inline-editable';
import { EditableStyleSelector } from '@/components/editable-style-selector';
import { DeleteRecipeDialog } from '@/components/delete-recipe-dialog';
import { recipesCollection } from '@/db';
import { getAvailableStyles } from '@/lib/actions/recipes';
import type { RecipeDocument } from '@/db';
import styles from '@/data/styles.json';

interface RecipeHeaderProps {
  recipe: RecipeDocument;
  showDelete?: boolean;
  redirectOnDelete?: boolean;
}

export function RecipeHeader({
  recipe,
  showDelete = true,
  redirectOnDelete = false,
}: RecipeHeaderProps) {
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
      {showDelete && (
        <div className="shrink-0">
          <DeleteRecipeDialog
            recipeId={recipe.id}
            recipeName={recipe.recipe.name}
            redirectOnDelete={redirectOnDelete}
          />
        </div>
      )}
    </div>
  );
}
