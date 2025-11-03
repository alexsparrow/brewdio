# AI Tools Implementation Summary

## Overview
This document outlines the AI tools implementation for the Brewdio homebrew application. The AI assistant can now help users with recipe management, ingredient searches, and brewing advice.

## Architecture

### 1. AI Tool Logging (`src/lib/ai-tools/logger.ts`)
Standardized logging utility for all AI tool executions:

- **`logToolStart(context)`** - Log when a tool begins execution
- **`logToolSuccess(context, result)`** - Log successful completion
- **`logToolError(context, error)`** - Log errors with stack traces
- **`withToolLogging(context, fn)`** - Wrapper function that automatically logs start, success, and errors

**Log Format:**
```
[AI Tool] {timestamp} [Recipe: {recipeId}] - {toolName} - {STATUS}
```

Example logs:
```
[AI Tool] 2025-10-30T12:34:56.789Z - searchHops - START { input: { search: 'cascade' } }
[AI Tool] 2025-10-30T12:34:56.790Z - searchHops - SUCCESS { result: [...] }
[AI Tool] 2025-10-30T12:34:56.791Z [Recipe: abc-123] - getCurrentRecipe - START
[AI Tool] 2025-10-30T12:34:56.792Z [Recipe: abc-123] - getCurrentRecipe - SUCCESS { result: {...} }
```

### 2. Centralized Actions (`src/lib/actions/recipes.ts`)
Reusable functions for recipe operations that can be used by both UI components and AI tools:

- **`createRecipe(params)`** - Create a new recipe with name, batch size, and beer style
- **`updateRecipe(params)`** - Update an existing recipe with BeerJSON data
- **`getAvailableStyles()`** - List all available beer styles

Note: Recipe retrieval uses TanStack DB's `useLiveQuery` hook in React components rather than a standalone action function.

### 3. AI Tools Organization

All tools include standardized logging with `withToolLogging()` wrapper.

#### Global Tools (`src/lib/ai-tools/global-tools.ts`)
Tools available in all contexts:

- **`searchFermentables`** - Search the fermentables database (grains, malts, sugars)
- **`searchHops`** - Search the hops database with alpha/beta acid info
- **`searchCultures`** - Search yeast cultures by name or producer
- **`listStyles`** - List all available beer styles
- **`createRecipe`** - Create a new recipe

#### Recipe Context Tools (`src/lib/ai-tools/recipe-tools.ts`)
Tools available only when viewing a specific recipe:

- **`getCurrentRecipe`** - Get the current recipe's BeerJSON data
- **`updateCurrentRecipe`** - Modify the current recipe in BeerJSON format

The recipe tools are created by the `createRecipeTools(currentRecipe?)` factory function:
- Accepts a `RecipeDocument` (retrieved via `useLiveQuery` in the chat sidebar)
- Returns an empty toolset when no recipe is provided
- Tools have direct access to the recipe data without database queries

### 4. Context-Aware Chat Sidebar

The `ChatSidebar` component (`src/components/chat-sidebar.tsx`) now:

- Accepts an optional `recipeId` prop
- Uses `useLiveQuery(recipesCollection)` to reactively fetch all recipes
- Finds the current recipe by ID from the query results
- Passes the recipe document directly to `createRecipeTools()`
- Dynamically loads appropriate tools based on context
- Shows a "Recipe Context Active" badge when in a recipe
- Displays context-specific welcome messages

**Architecture Benefits:**
- Respects TanStack DB layer - no direct Dexie access
- Reactive - tools automatically update when recipe changes
- No unnecessary database queries in tool execution
- Recipe data passed through React's data flow

### 5. Route Integration

The root component (`src/routes/__root.tsx`) extracts the current recipeId from the route and passes it to the ChatSidebar, making it automatically context-aware.

## Logging and Debugging

All AI tool executions are logged with a consistent format to help with debugging and monitoring:

1. **Console Logs**: Check browser console for detailed execution logs
2. **Timestamps**: All logs include ISO timestamps
3. **Context**: Recipe-specific tools include the recipe ID in logs
4. **Input Parameters**: Tool inputs are logged at START
5. **Results**: Tool outputs are logged at SUCCESS
6. **Errors**: Full error messages and stack traces are logged at ERROR

To add logging to a new tool:
```typescript
execute: async (params) => {
  return withToolLogging(
    { toolName: 'myNewTool', input: params, recipeId: optionalRecipeId },
    async () => {
      // Your tool logic here
      return result;
    }
  );
}
```

## Usage

### For Users

**On the homepage:**
- Create new recipes
- Search for ingredients
- Ask about beer styles and brewing techniques
- Tools: `searchFermentables`, `searchHops`, `searchCultures`, `listStyles`, `createRecipe`

**In a recipe page:**
- All global tools remain available
- Additionally: View and modify the current recipe
- Additional tools: `getCurrentRecipe`, `updateCurrentRecipe`
- The AI automatically knows which recipe you're working on

### For Developers

**Adding a new global tool:**
1. Add the tool to `src/lib/ai-tools/global-tools.ts`
2. Define the schema with zod
3. Implement the execute function

**Adding a new recipe-context tool:**
1. Add the tool to the `createRecipeTools()` function in `src/lib/ai-tools/recipe-tools.ts`
2. Access the recipe data via `currentRecipe.recipe` (BeerJSON) or `currentRecipe.id`
3. Wrap execution with `withToolLogging()` for consistent logging
4. The tool will automatically be available only when a recipe is loaded

**Creating a new centralized action:**
1. Add the function to `src/lib/actions/recipes.ts`
2. Use it in both UI components and AI tools

## Example AI Interactions

### Global Context Examples:
- "Search for cascade hops"
- "What fermentables have 'pale' in the name?"
- "Create a new IPA recipe called 'Tropical Thunder' with 5 gallon batch size"
- "List all available beer styles"

### Recipe Context Examples:
- "Show me the current recipe"
- "Add cascade hops to this recipe"
- "Change the batch size to 10 gallons"
- "What's the current ABV of this recipe?"

## Benefits

1. **Separation of Concerns**: Actions are separate from UI and AI implementations
2. **Context Awareness**: Tools automatically adapt to the current page
3. **Type Safety**: Full TypeScript support with zod schemas
4. **Maintainability**: Easy to add new tools and actions
5. **Reusability**: Actions can be used by UI components, AI tools, and future integrations
