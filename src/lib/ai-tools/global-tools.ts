import type { ToolSet } from 'ai';
import z from 'zod';
import fermentables from '@/data/fermentables.json';
import hops from '@/data/hops.json';
import cultures from '@/data/cultures.json';
import { createRecipe, getAvailableStyles } from '@/lib/actions/recipes';
import { withToolLogging } from './logger';

/**
 * Global-level AI tools that don't require a specific recipe context.
 * These include database searches and recipe creation.
 */
export const globalTools: ToolSet = {
  searchFermentables: {
    description: 'Search for fermentables (grains, malts, sugars) usable in a beer recipe. Returns fermentables matching the search term.',
    inputSchema: z.object({
      search: z.string().describe('Search term to filter fermentables by name'),
    }),
    execute: async ({ search }: { search: string }) => {
      return withToolLogging(
        { toolName: 'searchFermentables', input: { search } },
        async () => {
          const result = fermentables.filter((f) =>
            f.name.toLowerCase().includes(search.toLowerCase())
          );
          return result;
        }
      );
    },
  },

  searchHops: {
    description: 'Search for hops usable in a beer recipe. Returns hops with their alpha acid, beta acid, and origin information.',
    inputSchema: z.object({
      search: z.string().describe('Search term to filter hops by name'),
    }),
    execute: async ({ search }: { search: string }) => {
      return withToolLogging(
        { toolName: 'searchHops', input: { search } },
        async () => {
          const result = hops.filter((h) =>
            h.name.toLowerCase().includes(search.toLowerCase())
          );
          return result;
        }
      );
    },
  },

  searchCultures: {
    description: 'Search for yeast cultures (ale, lager, etc) usable in a beer recipe. Returns cultures with their type, form, attenuation, and producer information.',
    inputSchema: z.object({
      search: z.string().describe('Search term to filter cultures by name or producer'),
    }),
    execute: async ({ search }: { search: string }) => {
      return withToolLogging(
        { toolName: 'searchCultures', input: { search } },
        async () => {
          const result = cultures.filter((c) =>
            c.name.toLowerCase().includes(search.toLowerCase()) ||
            (c.producer && c.producer.toLowerCase().includes(search.toLowerCase()))
          );
          return result;
        }
      );
    },
  },

  listStyles: {
    description: 'List all available beer styles that can be used when creating a recipe.',
    inputSchema: z.object({}),
    execute: async () => {
      return withToolLogging(
        { toolName: 'listStyles' },
        async () => {
          return getAvailableStyles();
        }
      );
    },
  },

  createRecipe: {
    description: 'Create a NEW beer recipe from scratch. This creates an entirely new recipe in the database. DO NOT use this to modify an existing recipe - use updateCurrentRecipe instead when in a recipe context.',
    inputSchema: z.object({
      name: z.string().describe('Name of the new recipe'),
      batchSize: z.number().describe('Batch size in gallons'),
      styleName: z.string().describe('Name of the beer style (must match exactly from listStyles)'),
    }),
    execute: async ({ name, batchSize, styleName }: { name: string; batchSize: number; styleName: string }) => {
      return withToolLogging(
        { toolName: 'createRecipe', input: { name, batchSize, styleName } },
        async () => {
          try {
            const recipeId = await createRecipe({ name, batchSize, styleName });
            return { success: true, recipeId, message: `Recipe "${name}" created successfully with ID: ${recipeId}` };
          } catch (error) {
            return { success: false, error: error instanceof Error ? error.message : 'Failed to create recipe' };
          }
        }
      );
    },
  },
};
