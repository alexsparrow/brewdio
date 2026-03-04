import { startSync as wasmStartSync, stopSync as wasmStopSync } from "brewdio-wasm";
import { getToken } from "@/lib/auth";

type SyncStatus = "disconnected" | "connecting" | "connected" | "client_outdated" | "server_outdated";
let status: SyncStatus = "disconnected";
let listeners = new Set<(s: SyncStatus) => void>();

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

function setStatus(s: SyncStatus) {
  status = s;
  for (const fn of listeners) fn(status);
}

export async function startSync(serverUrl: string) {
  setStatus("connecting");

  if (isTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    const { listen } = await import("@tauri-apps/api/event");
    listen<SyncStatus>("sync-status", (event) => setStatus(event.payload));
    await invoke("start_sync", { url: serverUrl });
  } else {
    // WASM mode: use the existing AppDb-based sync
    const { getAppDb } = await import("@/lib/db/wasm-app-db");
    const db = getAppDb();
    const token = getToken();
    let url = serverUrl;
    if (token) {
      url += (url.includes("?") ? "&" : "?") + `token=${token}`;
    }
    wasmStartSync(db, url, (s: SyncStatus) => setStatus(s));
  }
}

export async function stopSync() {
  if (isTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("stop_sync");
  } else {
    wasmStopSync();
  }
  setStatus("disconnected");
}

export function getSyncStatus(): SyncStatus {
  return status;
}

export function onSyncStatusChange(listener: (status: SyncStatus) => void): () => void {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}
