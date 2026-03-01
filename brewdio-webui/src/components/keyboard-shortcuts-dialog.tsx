import { useState, useCallback } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useCommandHandler } from "@/hooks/use-command";
import { useIsMac, formatShortcut } from "@/hooks/use-platform";
import { getCommandsByGroup } from "@/lib/commands/commands";

export function KeyboardShortcutsDialog() {
  const [open, setOpen] = useState(false);
  const isMac = useIsMac();

  useCommandHandler(
    "shortcuts.show",
    useCallback(() => setOpen(true), [])
  );

  const groupedCommands = getCommandsByGroup();

  // Only show commands that have shortcuts
  const groupsWithShortcuts = Array.from(groupedCommands.entries())
    .map(([group, cmds]) => [group, cmds.filter((c) => c.shortcut)] as const)
    .filter(([, cmds]) => cmds.length > 0);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-[500px] max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Keyboard Shortcuts</DialogTitle>
          <DialogDescription>
            Available keyboard shortcuts for brewdio.
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-6 py-4">
          {groupsWithShortcuts.map(([group, cmds]) => (
            <div key={group}>
              <h3 className="text-sm font-semibold text-muted-foreground mb-3">
                {group}
              </h3>
              <div className="space-y-2">
                {cmds.map((cmd) => (
                  <div
                    key={cmd.id}
                    className="flex items-center justify-between py-1"
                  >
                    <div className="flex items-center gap-2">
                      {cmd.icon && <cmd.icon className="h-4 w-4 text-muted-foreground" />}
                      <span className="text-sm">{cmd.label}</span>
                    </div>
                    <ShortcutBadge shortcut={cmd.shortcut!} isMac={isMac} />
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}

function ShortcutBadge({ shortcut, isMac }: { shortcut: string; isMac: boolean }) {
  const formatted = formatShortcut(shortcut, isMac);

  if (isMac) {
    // On macOS, show modifier symbols individually in kbd tags
    const parts = shortcut.split("+");
    return (
      <div className="flex items-center gap-0.5">
        {parts.map((part, i) => {
          const p = part.trim();
          let display: string;
          if (p === "$mod") display = "\u2318";
          else if (p === "Shift") display = "\u21E7";
          else if (p === "Alt") display = "\u2325";
          else display = p.length === 1 ? p.toUpperCase() : p;

          return (
            <kbd
              key={i}
              className="inline-flex h-5 min-w-5 items-center justify-center rounded border bg-muted px-1 text-[10px] font-medium text-muted-foreground"
            >
              {display}
            </kbd>
          );
        })}
      </div>
    );
  }

  // On other platforms, show formatted string in a single kbd
  return (
    <kbd className="inline-flex h-5 items-center justify-center rounded border bg-muted px-1.5 text-[10px] font-medium text-muted-foreground">
      {formatted}
    </kbd>
  );
}
