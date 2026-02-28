import type { AppDb as AppDbClass } from 'brewdio-wasm';
import type { QueryClient } from '@tanstack/react-query';

// ---------------------------------------------------------------------------
// Singleton AppDb instance
// ---------------------------------------------------------------------------

let appDb: AppDbClass | null = null;

/** Call once during app initialisation (before React renders). */
export function initAppDb(db: AppDbClass) {
  appDb = db;
}

export function getAppDb(): AppDbClass {
  if (!appDb) throw new Error('AppDb not initialised – call initAppDb() first');
  return appDb;
}

/**
 * Wire up the WASM change-notification callbacks so that external mutations
 * (e.g. incoming sync) automatically invalidate the TanStack Query cache.
 */
export function registerChangeCallback(queryClient: QueryClient) {
  const db = getAppDb();
  db.onRecipesChange(() => {
    queryClient.invalidateQueries({ queryKey: ['recipes'] });
  });
  db.onBatchesChange(() => {
    queryClient.invalidateQueries({ queryKey: ['batches'] });
  });
  db.onSettingsChange(() => {
    queryClient.invalidateQueries({ queryKey: ['settings'] });
  });
  db.onEquipmentChange(() => {
    queryClient.invalidateQueries({ queryKey: ['equipment-profiles'] });
  });
}
