import type { AppDb } from "brewdio-wasm";

let appDb: AppDb | null = null;

export function setAppDb(db: AppDb) {
  appDb = db;
}

export function getAppDb(): AppDb {
  if (!appDb) throw new Error("AppDb not initialised – call setAppDb() first");
  return appDb;
}
