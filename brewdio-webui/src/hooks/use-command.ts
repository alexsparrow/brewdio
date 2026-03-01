import { useEffect, useMemo } from "react";
import { useRouterState } from "@tanstack/react-router";
import { registerHandler, type CommandContext } from "@/lib/commands/command-registry";
import { commands } from "@/lib/commands/commands";

/**
 * Register a command handler that is active while the component is mounted.
 */
export function useCommandHandler(id: string, handler: () => void): void {
  useEffect(() => {
    return registerHandler(id, handler);
  }, [id, handler]);
}

/**
 * Derive the current CommandContext[] from the route.
 */
export function useCommandContext(): CommandContext[] {
  const routerState = useRouterState();
  const pathname = routerState.location.pathname;

  return useMemo(() => {
    const contexts: CommandContext[] = ["global"];

    if (pathname === "/") {
      contexts.push("home");
    } else if (pathname.startsWith("/recipes/")) {
      contexts.push("recipe-detail");
    } else if (pathname.startsWith("/batches/")) {
      contexts.push("batch-detail");
    } else if (pathname === "/settings") {
      contexts.push("settings");
    }

    return contexts;
  }, [pathname]);
}

/**
 * Return commands available in the current route context.
 */
export function useAvailableCommands() {
  const contexts = useCommandContext();

  return useMemo(() => {
    return commands.filter((cmd) =>
      cmd.contexts.some((ctx) => contexts.includes(ctx))
    );
  }, [contexts]);
}
