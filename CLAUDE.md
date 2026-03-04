
# Brewdio

Brewing recipe manager with offline-first sync. Monorepo with Rust backend crates, a React web UI, and a Tauri desktop app.

## Project structure

```
brewdio-core/         — Shared types (BeerJSON, calculations, equipment profiles)
brewdio-persistence/  — SQLite storage, Automerge CRDT sync, batch/recipe/settings CRUD
brewdio-wasm/         — WASM bindings (brewdio-core compiled to WebAssembly for the web UI)
brewdio-server/       — Sync server (WebSocket relay + auth + user DB)
brewdio-tui/          — Terminal UI (ratatui)
brewdio-desktop/      — Tauri v2 desktop app (wraps brewdio-webui with native SQLite backend)
brewdio-webui/        — React + Vite frontend (runs standalone via WASM or inside Tauri)
```

Cargo workspace members: `brewdio-core`, `brewdio-persistence`, `brewdio-wasm`, `brewdio-tui`, `brewdio-server`, `brewdio-desktop`. Note: `brewdio-desktop` is excluded from `default-members` — use `-p brewdio-desktop` or `cargo tauri` commands explicitly.

Bun workspaces: `brewdio-webui`, `brewdio-wasm/pkg`.

## Bun

Default to using Bun instead of Node.js.

- Use `bun <file>` instead of `node <file>` or `ts-node <file>`
- Use `bun test` instead of `jest` or `vitest`
- Use `bun build <file.html|file.ts|file.css>` instead of `webpack` or `esbuild`
- Use `bun install` instead of `npm install` or `yarn install` or `pnpm install`
- Use `bun run <script>` instead of `npm run <script>` or `yarn run <script>` or `pnpm run <script>`
- Bun automatically loads .env, so don't use dotenv.

### APIs

- `Bun.serve()` supports WebSockets, HTTPS, and routes. Don't use `express`.
- `bun:sqlite` for SQLite. Don't use `better-sqlite3`.
- `Bun.redis` for Redis. Don't use `ioredis`.
- `Bun.sql` for Postgres. Don't use `pg` or `postgres.js`.
- `WebSocket` is built-in. Don't use `ws`.
- Prefer `Bun.file` over `node:fs`'s readFile/writeFile
- Bun.$`ls` instead of execa.

## Type checking

- `bun run typecheck` from the repo root runs typecheck across all workspaces
- `bun run typecheck` from `brewdio-webui/` runs `tsc -b`
- `cargo check -p brewdio-desktop` checks the Tauri desktop app
- `cargo test -p brewdio-persistence` runs all persistence tests (39 tests)

## Testing

Use `bun test` to run tests.

```ts#index.test.ts
import { test, expect } from "bun:test";

test("hello world", () => {
  expect(1).toBe(1);
});
```

## Desktop app (Tauri v2)

The desktop app lives in `brewdio-desktop/`. It wraps `brewdio-webui` in a Tauri webview and uses native SQLite (via `brewdio-persistence` with the `native` feature) instead of IndexedDB/WASM.

### Key files

- `brewdio-desktop/tauri.conf.json` — Tauri config (app identity, window size, bundle settings, build commands)
- `brewdio-desktop/src/main.rs` — App setup, DB initialization, Tauri command registration
- `brewdio-desktop/src/commands.rs` — All `#[tauri::command]` handlers (recipes, batches, settings, equipment, sync)
- `brewdio-desktop/capabilities/default.json` — Tauri permission capabilities

### Architecture

The web UI detects whether it's running inside Tauri (via `window.__TAURI_INTERNALS__`) and swaps the storage backend:
- **Web**: Uses WASM (`brewdio-wasm`) + IndexedDB
- **Tauri**: Uses `@tauri-apps/api/core.invoke()` to call Rust commands that operate on native SQLite

The desktop app shares the same SQLite database path as the TUI (via the `directories` crate: `~/Library/Application Support/com.brewdio.brewdio/` on macOS).

### Running locally

```sh
# Prerequisites: build WASM first (needed by the web UI)
bun run build:wasm

# Dev mode (launches Tauri dev window with HMR)
bun run tauri:dev

# Production build
bun run tauri:build
```

Or using cargo directly:
```sh
cargo tauri dev --config brewdio-desktop/tauri.conf.json
cargo tauri build --config brewdio-desktop/tauri.conf.json
```

### Tauri commands

All commands are defined in `brewdio-desktop/src/commands.rs`. They use `AppState` (holds `Arc<Mutex<rusqlite::Connection>>`) and emit `db-change` events to notify the frontend of mutations. Available commands:

- Recipes: `list_recipes`, `get_recipe`, `create_recipe`, `update_recipe`, `set_recipe_equipment`, `delete_recipe`
- Batches: `list_batches`, `get_batch`, `create_batch`, `update_batch`, `delete_batch`
- Settings: `get_settings`, `save_settings`
- Equipment: `list_equipment_profiles`, `create_equipment_profile`, `update_equipment_profile`, `delete_equipment_profile`
- Sync: `start_sync`, `stop_sync`

### CI/CD

Desktop builds (`.dmg` for macOS ARM, `.AppImage` for Linux x86-64) are built and attached to GitHub releases via the `build-desktop` job in `.github/workflows/release-please.yml`. Versions in both `brewdio-desktop/Cargo.toml` and `brewdio-desktop/tauri.conf.json` are bumped automatically by release-please.

## Frontend

The web UI uses Vite (see `brewdio-webui/vite.config.ts`). For standalone Bun projects, use HTML imports with `Bun.serve()`:

```ts#index.ts
import index from "./index.html"

Bun.serve({
  routes: {
    "/": index,
    "/api/users/:id": {
      GET: (req) => {
        return new Response(JSON.stringify({ id: req.params.id }));
      },
    },
  },
  websocket: {
    open: (ws) => {
      ws.send("Hello, world!");
    },
    message: (ws, message) => {
      ws.send(message);
    },
    close: (ws) => {
      // handle close
    }
  },
  development: {
    hmr: true,
    console: true,
  }
})
```

For more information, read the Bun API docs in `node_modules/bun-types/docs/**.md`.
