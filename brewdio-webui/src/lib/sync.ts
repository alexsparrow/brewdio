import { startSync as wasmStartSync, stopSync as wasmStopSync } from "brewdio-wasm";
import { getAppDb } from "@/lib/db/app-db";

type SyncStatus = "disconnected" | "connecting" | "connected";
let status: SyncStatus = "disconnected";
let listeners = new Set<(s: SyncStatus) => void>();

function setStatus(s: SyncStatus) {
  status = s;
  for (const fn of listeners) fn(status);
}

export function startSync(serverUrl: string) {
  const db = getAppDb();
  setStatus("connecting");
  wasmStartSync(db, serverUrl, (s: SyncStatus) => setStatus(s));
}

export function stopSync() {
  wasmStopSync();
  setStatus("disconnected");
}

export function getSyncStatus(): SyncStatus {
  return status;
}

export function onSyncStatusChange(listener: (status: SyncStatus) => void): () => void {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}
