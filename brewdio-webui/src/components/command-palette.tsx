import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useRecipes } from "@/lib/db/recipes";
import { useBatches } from "@/lib/db/batches";
import { useAvailableCommands, useCommandHandler } from "@/hooks/use-command";
import { useIsMac, formatShortcut } from "@/hooks/use-platform";
import { executeCommand, hasHandler } from "@/lib/commands/command-registry";
import {
  CommandDialog,
  CommandInput,
  CommandList,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandShortcut,
} from "@/components/ui/command";
import { Beaker, FileText } from "lucide-react";

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();
  const isMac = useIsMac();
  const availableCommands = useAvailableCommands();
  const { data: recipes } = useRecipes();
  const { data: batches } = useBatches();

  // Register the palette.open command handler
  useCommandHandler(
    "palette.open",
    useCallback(() => setOpen(true), [])
  );

  // Close palette on escape (handled by CommandDialog/Dialog)
  // Close palette on navigation
  useEffect(() => {
    if (open) {
      const handleNav = () => setOpen(false);
      window.addEventListener("popstate", handleNav);
      return () => window.removeEventListener("popstate", handleNav);
    }
  }, [open]);

  const handleSelect = (commandId: string) => {
    setOpen(false);
    // Use requestAnimationFrame so the dialog closes before the command runs
    // (important for commands that open other dialogs)
    requestAnimationFrame(() => {
      executeCommand(commandId);
    });
  };

  const handleRecipeSelect = (recipeId: string) => {
    setOpen(false);
    navigate({ to: "/recipes/$recipeId", params: { recipeId } });
  };

  const handleBatchSelect = (batchId: string) => {
    setOpen(false);
    navigate({ to: "/batches/$batchId", params: { batchId } });
  };

  // Group commands by their group name, excluding palette.open itself
  const groupedCommands = new Map<string, typeof availableCommands>();
  for (const cmd of availableCommands) {
    if (cmd.id === "palette.open") continue;
    const group = groupedCommands.get(cmd.group) ?? [];
    group.push(cmd);
    groupedCommands.set(cmd.group, group);
  }

  return (
    <CommandDialog
      open={open}
      onOpenChange={setOpen}
      title="Command Palette"
      description="Search commands, recipes, and batches..."
      showCloseButton={false}
    >
      <CommandInput placeholder="Type a command or search..." />
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>

        {/* Recipes */}
        {recipes && recipes.length > 0 && (
          <CommandGroup heading="Recipes">
            {recipes.map((r) => (
              <CommandItem
                key={r.id}
                value={`recipe: ${r.recipe.name}`}
                onSelect={() => handleRecipeSelect(r.id)}
              >
                <FileText className="mr-2 h-4 w-4" />
                <span>{r.recipe.name}</span>
                {r.recipe.style && (
                  <span className="ml-2 text-xs text-muted-foreground">
                    {r.recipe.style.name}
                  </span>
                )}
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        {/* Batches */}
        {batches && batches.length > 0 && (
          <CommandGroup heading="Batches">
            {batches.slice(0, 5).map((b) => (
              <CommandItem
                key={b.id}
                value={`batch: ${b.name}`}
                onSelect={() => handleBatchSelect(b.id)}
              >
                <Beaker className="mr-2 h-4 w-4" />
                <span>{b.name}</span>
                <span className="ml-2 text-xs text-muted-foreground">
                  {b.recipe.name}
                </span>
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        {/* Commands grouped */}
        {Array.from(groupedCommands.entries()).map(([group, cmds]) => (
          <CommandGroup key={group} heading={group}>
            {cmds.map((cmd) => {
              const Icon = cmd.icon;
              const isAvailable = cmd.contexts.includes("global") || hasHandler(cmd.id);
              return (
                <CommandItem
                  key={cmd.id}
                  value={cmd.label}
                  onSelect={() => handleSelect(cmd.id)}
                  disabled={!isAvailable}
                >
                  {Icon && <Icon className="mr-2 h-4 w-4" />}
                  <span>{cmd.label}</span>
                  {cmd.shortcut && (
                    <CommandShortcut>
                      {formatShortcut(cmd.shortcut, isMac)}
                    </CommandShortcut>
                  )}
                </CommandItem>
              );
            })}
          </CommandGroup>
        ))}
      </CommandList>
    </CommandDialog>
  );
}
