import type { ToolSet } from 'ai';
import z from 'zod';
import type { RecipeType } from '@beerjson/beerjson';
import { updateRecipe } from '@/lib/actions/recipes';
import { withToolLogging } from './logger';
import type { RecipeDocument } from '@/db';

/**
 * Create recipe-context AI tools that operate on a specific recipe.
 * When currentRecipe is provided, the tools will automatically use that recipe's data.
 *
 * @param currentRecipe - Optional recipe document for context-aware tools
 */
export function createRecipeTools(currentRecipe?: RecipeDocument): ToolSet {
  // If we have a current recipe context, create tools that implicitly use it
  if (currentRecipe) {
    return {
      getCurrentRecipe: {
        description: 'Get the CURRENT recipe that the user is viewing. Use this to read the recipe data before making modifications. Returns the full BeerJSON recipe data including all ingredients.',
        inputSchema: z.object({}),
        execute: async () => {
          return withToolLogging(
            { toolName: 'getCurrentRecipe', recipeId: currentRecipe.id },
            async () => {
              return { success: true, recipe: currentRecipe.recipe };
            }
          );
        },
      },

      updateCurrentRecipe: {
        description: 'Update the CURRENT recipe that the user is viewing. Use this to modify the recipe by providing the complete updated BeerJSON data. IMPORTANT: Always call getCurrentRecipe first to get the existing recipe, then modify it, then pass the complete modified recipe to this tool. This replaces the entire recipe with your modifications.',
        inputSchema: z.object({
          recipe: z.any().describe('Complete BeerJSON RecipeType object with ALL fields including your modifications. Must include name, batch_size, ingredients (fermentable_additions, hop_additions, culture_additions), style, efficiency, etc.'),
        }),
        execute: async ({ recipe }: { recipe: RecipeType }) => {
          return withToolLogging(
            { toolName: 'updateCurrentRecipe', recipeId: currentRecipe.id, input: { recipeName: recipe?.name || 'unknown' } },
            async () => {
              try {
                // Basic validation before passing to update
                if (!recipe) {
                  return {
                    success: false,
                    error: 'Recipe data is missing. You must provide a complete recipe object.'
                  };
                }

                if (!recipe.name || !recipe.batch_size || !recipe.ingredients) {
                  return {
                    success: false,
                    error: 'Recipe is incomplete. It must have: name, batch_size, and ingredients. Please call getCurrentRecipe first to get the complete recipe structure, then modify it.'
                  };
                }

                await updateRecipe({ recipeId: currentRecipe.id, recipe });
                return {
                  success: true,
                  message: `Recipe "${recipe.name}" updated successfully`
                };
              } catch (error) {
                return {
                  success: false,
                  error: `Failed to update recipe: ${error instanceof Error ? error.message : 'Unknown error'}`
                };
              }
            }
          );
        },
      },
    };
  }

  // If no current recipe context, return empty tools (these won't be available)
  return {};
}
