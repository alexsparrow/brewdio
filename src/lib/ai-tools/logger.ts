/**
 * Logging utility for AI tools
 * Provides consistent formatting for tool execution logging
 */

export interface ToolLogContext {
  toolName: string;
  input?: Record<string, any>;
  recipeId?: string;
}

export interface ToolLogResult {
  success: boolean;
  error?: string;
  data?: any;
}

/**
 * Log when a tool starts executing
 */
export function logToolStart(context: ToolLogContext): void {
  const timestamp = new Date().toISOString();
  const recipeContext = context.recipeId ? ` [Recipe: ${context.recipeId}]` : '';

  console.log(
    `[AI Tool] ${timestamp}${recipeContext} - ${context.toolName} - START`,
    context.input ? { input: context.input } : ''
  );
}

/**
 * Log when a tool completes successfully
 */
export function logToolSuccess(context: ToolLogContext, result: any): void {
  const timestamp = new Date().toISOString();
  const recipeContext = context.recipeId ? ` [Recipe: ${context.recipeId}]` : '';

  console.log(
    `[AI Tool] ${timestamp}${recipeContext} - ${context.toolName} - SUCCESS`,
    { result }
  );
}

/**
 * Log when a tool encounters an error
 */
export function logToolError(context: ToolLogContext, error: any): void {
  const timestamp = new Date().toISOString();
  const recipeContext = context.recipeId ? ` [Recipe: ${context.recipeId}]` : '';
  const errorMessage = error instanceof Error ? error.message : String(error);

  console.error(
    `[AI Tool] ${timestamp}${recipeContext} - ${context.toolName} - ERROR`,
    { error: errorMessage, stack: error instanceof Error ? error.stack : undefined }
  );
}

/**
 * Wrapper function to automatically log tool execution
 */
export async function withToolLogging<T>(
  context: ToolLogContext,
  fn: () => Promise<T>
): Promise<T> {
  logToolStart(context);

  try {
    const result = await fn();
    logToolSuccess(context, result);
    return result;
  } catch (error) {
    logToolError(context, error);
    throw error;
  }
}
