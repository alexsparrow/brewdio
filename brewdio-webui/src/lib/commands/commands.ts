import {
  Home,
  Settings2,
  PanelLeftIcon,
  Search,
  Keyboard,
  Sun,
  Plus,
  Beaker,
  Wheat,
  Leaf,
  FlaskConical,
  Code,
  StickyNote,
} from "lucide-react";
import type { Command } from "./command-registry";

export const commands: Command[] = [
  // General
  {
    id: "palette.open",
    label: "Command Palette",
    group: "General",
    shortcut: "$mod+k",
    icon: Search,
    contexts: ["global"],
  },
  {
    id: "shortcuts.show",
    label: "Keyboard Shortcuts",
    group: "General",
    shortcut: "$mod+/",
    icon: Keyboard,
    contexts: ["global"],
  },
  {
    id: "sidebar.toggle",
    label: "Toggle Sidebar",
    group: "General",
    shortcut: "$mod+b",
    icon: PanelLeftIcon,
    contexts: ["global"],
  },
  {
    id: "theme.toggle",
    label: "Toggle Theme",
    group: "General",
    shortcut: "$mod+Shift+t",
    icon: Sun,
    contexts: ["global"],
  },

  // Navigation
  {
    id: "navigate.home",
    label: "Go Home",
    group: "Navigation",
    shortcut: "$mod+Shift+h",
    icon: Home,
    contexts: ["global"],
  },
  {
    id: "navigate.settings",
    label: "Settings",
    group: "Navigation",
    shortcut: "$mod+,",
    icon: Settings2,
    contexts: ["global"],
  },

  // Recipes
  {
    id: "recipe.create",
    label: "Create New Recipe",
    group: "Recipes",
    icon: Plus,
    contexts: ["global"],
  },
  {
    id: "recipe.brew",
    label: "Brew Batch",
    group: "Recipes",
    shortcut: "$mod+Shift+b",
    icon: Beaker,
    contexts: ["recipe-detail"],
  },
  {
    id: "recipe.jsonEditor",
    label: "JSON Editor",
    group: "Recipes",
    shortcut: "$mod+Shift+j",
    icon: Code,
    contexts: ["recipe-detail", "batch-detail"],
  },

  // Ingredients
  {
    id: "recipe.addFermentable",
    label: "Add Fermentable",
    group: "Ingredients",
    shortcut: "$mod+Shift+f",
    icon: Wheat,
    contexts: ["recipe-detail", "batch-detail"],
  },
  {
    id: "recipe.addHop",
    label: "Add Hop",
    group: "Ingredients",
    shortcut: "$mod+Shift+o",
    icon: Leaf,
    contexts: ["recipe-detail", "batch-detail"],
  },
  {
    id: "recipe.addCulture",
    label: "Add Culture",
    group: "Ingredients",
    shortcut: "$mod+Shift+y",
    icon: FlaskConical,
    contexts: ["recipe-detail", "batch-detail"],
  },

  // View / Scroll
  {
    id: "recipe.scrollFermentables",
    label: "Go to Fermentables",
    group: "View",
    icon: Wheat,
    contexts: ["recipe-detail", "batch-detail"],
  },
  {
    id: "recipe.scrollHops",
    label: "Go to Hops",
    group: "View",
    icon: Leaf,
    contexts: ["recipe-detail", "batch-detail"],
  },
  {
    id: "recipe.scrollCultures",
    label: "Go to Cultures",
    group: "View",
    icon: FlaskConical,
    contexts: ["recipe-detail", "batch-detail"],
  },
  {
    id: "recipe.scrollNotes",
    label: "Go to Notes",
    group: "View",
    shortcut: "$mod+Shift+n",
    icon: StickyNote,
    contexts: ["recipe-detail", "batch-detail"],
  },
];

export function getCommandById(id: string): Command | undefined {
  return commands.find((c) => c.id === id);
}

export function getCommandsByGroup(): Map<string, Command[]> {
  const groups = new Map<string, Command[]>();
  for (const cmd of commands) {
    const group = groups.get(cmd.group) ?? [];
    group.push(cmd);
    groups.set(cmd.group, group);
  }
  return groups;
}
