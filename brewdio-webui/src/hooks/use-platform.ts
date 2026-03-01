import { useMemo } from "react";

export function useIsMac(): boolean {
  return useMemo(() => {
    if (typeof navigator === "undefined") return false;
    return /Mac|iPhone|iPad/.test(navigator.platform);
  }, []);
}

/**
 * Formats a tinykeys shortcut string for display.
 * "$mod+Shift+f" -> "⇧⌘F" (mac) or "Ctrl+Shift+F" (other)
 */
export function formatShortcut(shortcut: string, isMac: boolean): string {
  const parts = shortcut.split("+");
  const result: string[] = [];

  for (const part of parts) {
    const p = part.trim();
    if (p === "$mod") {
      result.push(isMac ? "\u2318" : "Ctrl");
    } else if (p === "Shift") {
      result.push(isMac ? "\u21E7" : "Shift");
    } else if (p === "Alt") {
      result.push(isMac ? "\u2325" : "Alt");
    } else {
      // Key name — capitalize for display
      result.push(p.length === 1 ? p.toUpperCase() : p);
    }
  }

  if (isMac) {
    return result.join("");
  }
  return result.join("+");
}
