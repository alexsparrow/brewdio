import { useQuery, type QueryClient } from '@tanstack/react-query';
import { getBackend } from './app-db';
import { settingsKeys } from './query-keys';
import type { SettingsDocument as BackendSettingsDocument } from './backend';

export { settingsKeys };

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
// Queries
// ---------------------------------------------------------------------------

export function useSettings() {
  return useQuery<SettingsDocument>({
    queryKey: settingsKeys.all,
    queryFn: async () => {
      const raw = await getBackend().getSettings() as unknown as SettingsDocument | null;
      return raw ?? DEFAULT_SETTINGS;
    },
  });
}

// ---------------------------------------------------------------------------
// Imperative helpers
// ---------------------------------------------------------------------------

export async function getSettingsImperative(): Promise<SettingsDocument> {
  const raw = await getBackend().getSettings() as unknown as SettingsDocument | null;
  return raw ?? DEFAULT_SETTINGS;
}

export async function saveSettingsImperative(settings: SettingsDocument): Promise<void> {
  return getBackend().saveSettings(settings as unknown as BackendSettingsDocument);
}

// ---------------------------------------------------------------------------
// Cache invalidation helper
// ---------------------------------------------------------------------------

export function invalidateSettings(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: settingsKeys.all });
}
