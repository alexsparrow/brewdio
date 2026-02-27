import { SyncWorker } from "brewdio-wasm";
import { getRecipeDb } from "@/lib/db/recipes";

type SyncStatus = "disconnected" | "connecting" | "connected";

let worker: SyncWorker | null = null;
let ws: WebSocket | null = null;
let dirtyInterval: ReturnType<typeof setInterval> | null = null;
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
let status: SyncStatus = "disconnected";
let listeners: Set<(status: SyncStatus) => void> = new Set();

function setStatus(s: SyncStatus) {
  status = s;
  for (const fn of listeners) fn(s);
}

function sendAll(messages: string[]) {
  if (!ws || ws.readyState !== WebSocket.OPEN) return;
  for (const msg of messages) ws.send(msg);
}

function cleanup() {
  if (dirtyInterval != null) {
    clearInterval(dirtyInterval);
    dirtyInterval = null;
  }
  if (reconnectTimeout != null) {
    clearTimeout(reconnectTimeout);
    reconnectTimeout = null;
  }
  if (ws) {
    ws.onopen = null;
    ws.onmessage = null;
    ws.onclose = null;
    ws.onerror = null;
    if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
      ws.close();
    }
    ws = null;
  }
  if (worker) {
    worker.onDisconnected();
  }
  setStatus("disconnected");
}

let currentUrl: string | null = null;

function connect(serverUrl: string) {
  cleanup();
  currentUrl = serverUrl;
  worker = new SyncWorker();
  setStatus("connecting");

  ws = new WebSocket(serverUrl);

  ws.onopen = () => {
    const db = getRecipeDb();
    const hello = worker!.onConnected(db);
    ws!.send(hello);
  };

  ws.onmessage = (event) => {
    const db = getRecipeDb();
    const data = event.data as string;
    const parsed = JSON.parse(data);

    if (parsed.type === "Hello") {
      console.log("[sync] received Hello from server");
      setStatus("connected");
      const replies: string[] = Array.from(worker!.processHello(db, data));
      console.log("[sync] sending", replies.length, "NewDoc replies");
      sendAll(replies);
      // Start dirty check interval
      if (dirtyInterval != null) clearInterval(dirtyInterval);
      dirtyInterval = setInterval(() => {
        const db = getRecipeDb();
        const msgs: string[] = Array.from(worker!.checkDirty(db));
        if (msgs.length > 0) console.log("[sync] dirty check: sending", msgs.length, "messages");
        sendAll(msgs);
      }, 2000);
    } else {
      console.log("[sync] received", parsed.type, "from server");
      const replies: string[] = Array.from(worker!.onMessage(db, data));
      if (replies.length > 0) console.log("[sync] sending", replies.length, "replies");
      sendAll(replies);
    }
  };

  ws.onclose = () => {
    const url = currentUrl;
    cleanup();
    if (url) {
      reconnectTimeout = setTimeout(() => connect(url), 5000);
    }
  };

  ws.onerror = () => {
    // onclose will fire after onerror, which handles reconnect
  };
}

export function startSync(serverUrl: string) {
  connect(serverUrl);
}

export function stopSync() {
  currentUrl = null;
  cleanup();
  if (worker) {
    worker.free();
    worker = null;
  }
}

export function getSyncStatus(): SyncStatus {
  return status;
}

export function onSyncStatusChange(listener: (status: SyncStatus) => void): () => void {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}
