import type { AppDb as AppDbClass } from 'brewdio-wasm';
import type { QueryClient } from '@tanstack/react-query';
import { recipeKeys, batchKeys, settingsKeys, equipmentKeys } from './query-keys';

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

// ---------------------------------------------------------------------------
// Cross-tab sync via BroadcastChannel
// ---------------------------------------------------------------------------

const CHANNEL_NAME = 'brewdio-db-changes';
let channel: BroadcastChannel | null = null;

type ChangeKind = typeof recipeKeys.all[0] | typeof batchKeys.all[0] | typeof settingsKeys.all[0] | typeof equipmentKeys.all[0];

function getBroadcastChannel(): BroadcastChannel | null {
  if (channel) return channel;
  if (typeof BroadcastChannel === 'undefined') return null;
  channel = new BroadcastChannel(CHANNEL_NAME);
  return channel;
}

function postChange(kind: ChangeKind) {
  getBroadcastChannel()?.postMessage(kind);
}

/**
 * Wire up the WASM change-notification callbacks so that external mutations
 * (e.g. incoming sync) automatically invalidate the TanStack Query cache.
 *
 * Also sets up a BroadcastChannel so that changes in one tab are reflected
 * in all other open tabs.
 */
export function registerChangeCallback(queryClient: QueryClient) {
  const db = getAppDb();
  db.onRecipesChange(() => {
    queryClient.invalidateQueries({ queryKey: recipeKeys.all });
    postChange('recipes');
  });
  db.onBatchesChange(() => {
    queryClient.invalidateQueries({ queryKey: batchKeys.all });
    postChange('batches');
  });
  db.onSettingsChange(() => {
    queryClient.invalidateQueries({ queryKey: settingsKeys.all });
    postChange('settings');
  });
  db.onEquipmentChange(() => {
    queryClient.invalidateQueries({ queryKey: equipmentKeys.all });
    postChange('equipment-profiles');
  });

  // Listen for changes from other tabs
  const bc = getBroadcastChannel();
  if (bc) {
    bc.onmessage = (event: MessageEvent<ChangeKind>) => {
      queryClient.invalidateQueries({ queryKey: [event.data] });
    };
  }
}
