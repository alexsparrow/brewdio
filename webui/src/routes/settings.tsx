import { createFileRoute } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { settingsCollection, DEFAULT_SETTINGS, type SettingsDocument } from "@/db";
import { useEffect, useState } from "react";
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

export const Route = createFileRoute("/settings")({
  component: SettingsComponent,
});

const volumeUnits = ["ml", "l", "tsp", "tbsp", "floz", "cup", "pt", "qt", "gal", "bbl"] as const;
const massUnits = ["mg", "g", "kg", "lb", "oz"] as const;
const temperatureUnits = ["C", "F"] as const;

function SettingsComponent() {
  const { data: allSettings, status } = useLiveQuery(settingsCollection);
  const [settings, setSettings] = useState<SettingsDocument>(DEFAULT_SETTINGS);
  const [hasChanges, setHasChanges] = useState(false);

  // Load settings or use defaults
  useEffect(() => {
    if (status === "ready" && allSettings) {
      const userSettings = allSettings.find((s) => s.id === "user-settings");
      if (userSettings) {
        setSettings(userSettings);
      } else {
        // Initialize with defaults
        settingsCollection.insert(DEFAULT_SETTINGS);
      }
    }
  }, [allSettings, status]);

  const handleSave = async () => {
    const existing = allSettings?.find((s) => s.id === "user-settings");

    if (existing) {
      await settingsCollection.update("user-settings", (draft) => {
        draft.vimMode = settings.vimMode;
        draft.defaultVolumeUnit = settings.defaultVolumeUnit;
        draft.defaultMassUnit = settings.defaultMassUnit;
        draft.defaultTemperatureUnit = settings.defaultTemperatureUnit;
        draft.openaiApiKey = settings.openaiApiKey;
        draft.defaultAuthor = settings.defaultAuthor;
      });
    } else {
      await settingsCollection.insert(settings);
    }

    setHasChanges(false);
  };

  const handleReset = () => {
    const userSettings = allSettings?.find((s) => s.id === "user-settings");
    if (userSettings) {
      setSettings(userSettings);
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

  if (status === "loading") {
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
    </div>
  );
}
