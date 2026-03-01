import type { LucideIcon } from "lucide-react";

export type CommandContext =
  | "global"
  | "home"
  | "recipe-detail"
  | "batch-detail"
  | "settings";

export interface Command {
  id: string;
  label: string;
  group: string;
  shortcut?: string;
  icon?: LucideIcon;
  contexts: CommandContext[];
}

// Module-level singleton handler map
const handlers = new Map<string, () => void>();

export function registerHandler(id: string, handler: () => void): () => void {
  handlers.set(id, handler);
  return () => {
    if (handlers.get(id) === handler) handlers.delete(id);
  };
}

export function executeCommand(id: string): boolean {
  const handler = handlers.get(id);
  if (handler) {
    handler();
    return true;
  }
  return false;
}

export function hasHandler(id: string): boolean {
  return handlers.has(id);
}
