import { useEffect } from "react";
import { tinykeys } from "tinykeys";
import { commands } from "@/lib/commands/commands";
import { executeCommand } from "@/lib/commands/command-registry";

function isEditableElement(el: EventTarget | null): boolean {
  if (!(el instanceof HTMLElement)) return false;
  const tag = el.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
  if (el.isContentEditable) return true;
  if (el.closest(".monaco-editor")) return true;
  return false;
}

export function KeyboardShortcutsProvider() {
  useEffect(() => {
    const bindings: Record<string, (e: KeyboardEvent) => void> = {};

    for (const cmd of commands) {
      if (!cmd.shortcut) continue;

      bindings[cmd.shortcut] = (e: KeyboardEvent) => {
        // Always allow command palette and shortcuts dialog, even in inputs
        const alwaysAllow = cmd.id === "palette.open" || cmd.id === "shortcuts.show";

        if (!alwaysAllow && isEditableElement(e.target)) {
          return;
        }

        e.preventDefault();
        executeCommand(cmd.id);
      };
    }

    const unsubscribe = tinykeys(window, bindings);
    return unsubscribe;
  }, []);

  return null;
}
