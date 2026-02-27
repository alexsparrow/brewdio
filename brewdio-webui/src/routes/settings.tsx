import { createFileRoute } from "@tanstack/react-router";
import { useSettings, saveSettingsImperative, DEFAULT_SETTINGS, type SettingsDocument } from "@/lib/db/settings";
import { useQueryClient } from "@tanstack/react-query";
import { settingsKeys } from "@/lib/db/settings";
import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { startSync, stopSync, getSyncStatus, onSyncStatusChange } from "@/lib/sync";

export const Route = createFileRoute("/settings")({
  component: SettingsComponent,
});

const volumeUnits = ["ml", "l", "tsp", "tbsp", "floz", "cup", "pt", "qt", "gal", "bbl"] as const;
const massUnits = ["mg", "g", "kg", "lb", "oz"] as const;
const temperatureUnits = ["C", "F"] as const;

function SettingsComponent() {
  const { data: currentSettings, status } = useSettings();
  const queryClient = useQueryClient();
  const [settings, setSettings] = useState<SettingsDocument>(DEFAULT_SETTINGS);
  const [hasChanges, setHasChanges] = useState(false);

  // Load settings when they become available
  useEffect(() => {
    if (status === "success" && currentSettings) {
      setSettings(currentSettings);
    }
  }, [currentSettings, status]);

  const handleSave = () => {
    saveSettingsImperative(settings);
    queryClient.invalidateQueries({ queryKey: settingsKeys.all });
    setHasChanges(false);
  };

  const handleReset = () => {
    if (currentSettings) {
      setSettings(currentSettings);
    } else {
      setSettings(DEFAULT_SETTINGS);
    }
    setHasChanges(false);
  };

  const updateSetting = <K extends keyof SettingsDocument>(
    key: K,
    value: SettingsDocument[K]
  ) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  if (status === "pending") {
    return <div>Loading settings...</div>;
  }

  return (
    <div className="flex flex-col gap-6 max-w-2xl">
      <div>
        <h1 className="text-3xl font-bold">Settings</h1>
        <p className="text-muted-foreground mt-2">
          Configure your preferences for the brewing app.
        </p>
      </div>

      <div className="space-y-6 border rounded-lg p-6">
        <div className="space-y-4">
          <h2 className="text-xl font-semibold">Recipe Defaults</h2>
          <p className="text-sm text-muted-foreground">
            These values will be used when creating new recipes.
          </p>

          <div className="space-y-2">
            <Label htmlFor="default-author">Default Author</Label>
            <Input
              id="default-author"
              type="text"
              value={settings.defaultAuthor || ""}
              onChange={(e) => updateSetting("defaultAuthor", e.target.value)}
              placeholder="Your name"
            />
            <p className="text-xs text-muted-foreground">
              This name will be used as the author for new recipes.
            </p>
          </div>
        </div>

        <div className="space-y-4">
          <h2 className="text-xl font-semibold">AI Assistant</h2>
          <p className="text-sm text-muted-foreground">
            Configure your AI model provider for the chat assistant.
          </p>

          <div className="space-y-2">
            <Label htmlFor="openai-key">OpenAI API Key</Label>
            <Input
              id="openai-key"
              type="password"
              value={settings.openaiApiKey || ""}
              onChange={(e) => updateSetting("openaiApiKey", e.target.value)}
              placeholder="sk-..."
            />
            <p className="text-xs text-muted-foreground">
              Your API key is stored locally and never sent to our servers.
            </p>
          </div>
        </div>

        <div className="space-y-4">
          <h2 className="text-xl font-semibold">Editor Preferences</h2>

          <div className="flex items-center space-x-2">
            <Checkbox
              id="vim-mode"
              checked={settings.vimMode}
              onCheckedChange={(checked) => updateSetting("vimMode", checked as boolean)}
            />
            <Label
              htmlFor="vim-mode"
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              Enable Vim mode in JSON editor
            </Label>
          </div>
        </div>

        <div className="space-y-4">
          <h2 className="text-xl font-semibold">Default Units</h2>
          <p className="text-sm text-muted-foreground">
            These units will be pre-selected when adding ingredients to recipes.
          </p>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="volume-unit">Volume</Label>
              <Select
                value={settings.defaultVolumeUnit}
                onValueChange={(value) =>
                  updateSetting("defaultVolumeUnit", value as SettingsDocument["defaultVolumeUnit"])
                }
              >
                <SelectTrigger id="volume-unit">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {volumeUnits.map((unit) => (
                    <SelectItem key={unit} value={unit}>
                      {unit}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="mass-unit">Mass</Label>
              <Select
                value={settings.defaultMassUnit}
                onValueChange={(value) =>
                  updateSetting("defaultMassUnit", value as SettingsDocument["defaultMassUnit"])
                }
              >
                <SelectTrigger id="mass-unit">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {massUnits.map((unit) => (
                    <SelectItem key={unit} value={unit}>
                      {unit}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="temperature-unit">Temperature</Label>
              <Select
                value={settings.defaultTemperatureUnit}
                onValueChange={(value) =>
                  updateSetting("defaultTemperatureUnit", value as SettingsDocument["defaultTemperatureUnit"])
                }
              >
                <SelectTrigger id="temperature-unit">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {temperatureUnits.map((unit) => (
                    <SelectItem key={unit} value={unit}>
                      {unit === "C" ? "Celsius (°C)" : "Fahrenheit (°F)"}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      </div>

      <div className="flex gap-3">
        <Button onClick={handleSave} disabled={!hasChanges}>
          {hasChanges ? "Save Changes" : "No Changes"}
        </Button>
        <Button variant="outline" onClick={handleReset} disabled={!hasChanges}>
          Reset
        </Button>
      </div>

      <SyncSection />
    </div>
  );
}

const statusColors: Record<string, string> = {
  connected: "bg-green-500",
  connecting: "bg-yellow-500",
  disconnected: "bg-gray-400",
};

function SyncSection() {
  const [syncStatus, setSyncStatus] = useState(getSyncStatus);
  const [serverUrl, setServerUrl] = useState(
    () => localStorage.getItem("brewdio_server") ?? ""
  );

  useEffect(() => {
    return onSyncStatusChange(setSyncStatus);
  }, []);

  const handleConnect = () => {
    if (!serverUrl.trim()) return;
    localStorage.setItem("brewdio_server", serverUrl.trim());
    startSync(serverUrl.trim());
  };

  const handleDisconnect = () => {
    localStorage.removeItem("brewdio_server");
    stopSync();
  };

  return (
    <div className="space-y-6 border rounded-lg p-6">
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Sync</h2>
        <p className="text-sm text-muted-foreground">
          Connect to a sync server to keep recipes, batches, and settings in
          sync across devices.
        </p>

        <div className="space-y-2">
          <Label htmlFor="sync-server-url">Server URL</Label>
          <Input
            id="sync-server-url"
            type="text"
            value={serverUrl}
            onChange={(e) => setServerUrl(e.target.value)}
            placeholder="ws://192.168.1.100:8080/ws"
            disabled={syncStatus !== "disconnected"}
          />
        </div>

        <div className="flex items-center gap-3">
          {syncStatus === "disconnected" ? (
            <Button onClick={handleConnect} disabled={!serverUrl.trim()}>
              Connect
            </Button>
          ) : (
            <Button variant="outline" onClick={handleDisconnect}>
              Disconnect
            </Button>
          )}
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <span
              className={`inline-block h-2.5 w-2.5 rounded-full ${statusColors[syncStatus]}`}
            />
            {syncStatus}
          </div>
        </div>
      </div>
    </div>
  );
}
