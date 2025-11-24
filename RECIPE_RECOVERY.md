# Recipe Recovery Guide

If your recipe gets corrupted by an AI update, here's how to recover:

## Symptoms of Corrupted Recipe
- Error: "can't access property 'fermentable_additions', beerRecipe.ingredients is undefined"
- Recipe page shows errors
- Unable to view or edit the recipe

## Recovery Options

### Option 1: Manual Fix via JSON Editor
1. Navigate to the recipe page
2. Click "Edit JSON" button
3. Ensure the recipe has this minimum structure:
```json
{
  "name": "Your Recipe Name",
  "type": "all grain",
  "author": "Brewdio User",
  "batch_size": {
    "value": 5.0,
    "unit": "gal"
  },
  "boil_size": {
    "value": 6.5,
    "unit": "gal"
  },
  "boil_time": {
    "value": 60,
    "unit": "min"
  },
  "efficiency": {
    "brewhouse": 72,
    "conversion": 100,
    "lauter": 95,
    "mash": 95
  },
  "style": { ... },
  "ingredients": {
    "fermentable_additions": [],
    "hop_additions": [],
    "culture_additions": []
  }
}
```
4. If `ingredients` is missing, add it with empty arrays
5. Save the JSON

### Option 2: Browser Console Fix
1. Open browser console (F12)
2. Navigate to the corrupted recipe page
3. Run this recovery script:
```javascript
// Get the recipe ID from the URL
const recipeId = window.location.pathname.split('/').pop();

// Import necessary functions
import('/@fs/Users/alex/projects/brewdio/src/db.ts').then(async ({ recipesCollection }) => {
  await recipesCollection.update(recipeId, (draft) => {
    // Ensure ingredients object exists
    if (!draft.recipe.ingredients) {
      draft.recipe.ingredients = {
        fermentable_additions: [],
        hop_additions: [],
        culture_additions: []
      };
    }
    // Ensure each array exists
    if (!draft.recipe.ingredients.fermentable_additions) {
      draft.recipe.ingredients.fermentable_additions = [];
    }
    if (!draft.recipe.ingredients.hop_additions) {
      draft.recipe.ingredients.hop_additions = [];
    }
    if (!draft.recipe.ingredients.culture_additions) {
      draft.recipe.ingredients.culture_additions = [];
    }
    draft.updatedAt = Date.now();
  });
  window.location.reload();
});
```

### Option 3: Delete and Recreate
If the recipe is too corrupted:
1. Delete the corrupted recipe from the recipes list
2. Create a new recipe with the same style
3. Re-add ingredients manually or ask the AI to help

## Prevention
The AI tools now have validation to prevent this issue:
- `updateRecipe` validates structure before saving
- `updateCurrentRecipe` tool checks for required fields
- Error messages guide the AI to call `getCurrentRecipe` first

## What Went Wrong
The AI likely:
1. Didn't call `getCurrentRecipe` first
2. Created a partial recipe object without all required fields
3. Passed an incomplete recipe to `updateCurrentRecipe`

The new validation prevents this by:
- Rejecting updates without `name`, `batch_size`, or `ingredients`
- Auto-creating empty ingredient arrays if missing
- Providing clear error messages to guide the AI
