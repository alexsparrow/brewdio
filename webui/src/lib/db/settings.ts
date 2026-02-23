import { useQuery, type QueryClient } from '@tanstack/react-query';
import { getRecipeDb } from './recipes';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface SettingsDocument {
  id: string;
  vimMode: boolean;
  defaultVolumeUnit: "ml" | "l" | "tsp" | "tbsp" | "floz" | "cup" | "pt" | "qt" | "gal" | "bbl";
  defaultMassUnit: "mg" | "g" | "kg" | "lb" | "oz";
  defaultTemperatureUnit: "C" | "F";
  openaiApiKey?: string;
  defaultAuthor?: string;
}

export const DEFAULT_SETTINGS: SettingsDocument = {
  id: "user-settings",
  vimMode: false,
  defaultVolumeUnit: "gal",
  defaultMassUnit: "lb",
  defaultTemperatureUnit: "F",
  openaiApiKey: "",
  defaultAuthor: "Brewdio User",
};

// ---------------------------------------------------------------------------
// Query keys
// ---------------------------------------------------------------------------

export const settingsKeys = {
  all: ['settings'] as const,
};

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

export function useSettings() {
  return useQuery<SettingsDocument>({
    queryKey: settingsKeys.all,
    queryFn: () => {
      const db = getRecipeDb();
      const raw = db.get_settings() as unknown as SettingsDocument | null;
      return raw ?? DEFAULT_SETTINGS;
    },
  });
}

// ---------------------------------------------------------------------------
// Imperative helpers
// ---------------------------------------------------------------------------

export function getSettingsImperative(): SettingsDocument {
  const db = getRecipeDb();
  const raw = db.get_settings() as unknown as SettingsDocument | null;
  return raw ?? DEFAULT_SETTINGS;
}

export function saveSettingsImperative(settings: SettingsDocument): void {
  const db = getRecipeDb();
  db.save_settings(settings as any);
}

// ---------------------------------------------------------------------------
// Cache invalidation helper
// ---------------------------------------------------------------------------

export function invalidateSettings(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: settingsKeys.all });
}
