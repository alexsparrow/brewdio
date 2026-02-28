# brewdio

> A **local-first** homebrewing app designed to be powerful and fun to use.

brewdio is a modern, privacy-focused brewing companion that puts you in control of your recipes and brew days. Everything runs locally—no accounts, no subscriptions. Optional multi-device sync via Automerge CRDTs.

---

## Features

### Recipe Management
- **BeerJSON Compliant** — Industry-standard recipe format ensures portability and future-proofing
- **Smart Calculations** — Automatic OG, FG, ABV, IBU, and color calculations using proven brewing formulas
- **Ingredient Database** — Comprehensive libraries of hops, fermentables, and yeast strains
- **Style Guidelines** — Built-in BJCP style ranges to help you hit your targets
- **Equipment Profiles** — Customize loss calculations for your specific mash tun, kettle, and fermenter

### Batch Tracking
- **Brew Day Companion** — Read-only batch views perfect for following along during brewing
- **Water Calculator** — Visual water requirement calculator with equipment-specific adjustments
- **Batch Notes** — Markdown-enabled notes for documenting your process and results

### Multi-Device Sync
- **Automerge CRDTs** — Conflict-free sync of recipes, batches, settings, and equipment profiles
- **Relay Server** — Lightweight Rust server relays changes between connected clients in real-time
- **Auto-Reconnect** — Clients automatically reconnect and resume sync after disconnects

### AI Brewing Assistant
- **Recipe Helper** — Chat-based assistant that can search ingredients, suggest styles, and create or modify recipes
- **Context-Aware** — Knows which recipe you're viewing and can read/update it directly
- **Tool-Based** — Uses structured tool calls for ingredient search, recipe creation, and editing
- **Private** — Runs against your own OpenAI API key; nothing leaves your browser except API calls

### User Experience
- **Local-First** — All data stored locally using SQLite (IndexedDB-backed in the browser, native on desktop)
- **Two Interfaces** — Full-featured web UI and terminal UI sharing the same data layer and sync protocol
- **Vim-Style TUI** — Navigate recipes, manage ingredients, and track batches entirely from the terminal with keyboard-driven workflows
- **Dark Mode** — Beautiful dark mode support in the web UI
- **Retro Dials** — Playful retro-cockpit gauge displays for calculated values
- **Live Vitals** — Real-time OG, FG, ABV, IBU, and SRM with color-coded BJCP style range bars (TUI and web)
- **Inline Editing** — Edit recipe values directly without cumbersome forms
- **JSON Editor** — Power users can edit raw BeerJSON with Monaco, optional Vim mode

---

## Architecture

brewdio is a Rust workspace with a shared core compiled to both WebAssembly and native targets:

```
brewdio/
├── brewdio-core/          # Brewing calculations, units, ingredient & style data
├── brewdio-persistence/   # SQLite storage & Automerge CRDT sync
│   ├── connection.rs          # Generic Connection trait
│   ├── connection_native.rs   # Native SQLite (rusqlite)
│   ├── connection_wasm.rs     # WASM SQLite (sqlite-wasm-rs + IndexedDB VFS)
│   ├── db.rs                  # Recipe CRUD + sync dispatch
│   ├── batch.rs               # Batch CRUD
│   ├── settings.rs            # User settings
│   ├── equipment_profile.rs   # Equipment profile CRUD
│   ├── sync.rs                # Automerge SyncSession wrapper
│   ├── protocol.rs            # Wire protocol (Hello, SyncDoc, NewDoc)
│   └── sync_worker.rs         # Native background sync worker (tokio)
├── brewdio-wasm/          # wasm-bindgen bindings (core + persistence + SyncWorker)
├── brewdio-server/        # Sync relay server (Axum + WebSocket + broadcast)
├── brewdio-tui/           # Terminal UI (ratatui)
└── brewdio-webui/         # React web frontend
    └── src/
        ├── routes/            # File-based routing (TanStack Router)
        ├── components/        # UI components (shadcn/ui)
        └── lib/
            ├── db/            # WASM database hooks & TanStack Query integration
            ├── sync.ts        # WebSocket sync transport layer
            ├── ai/            # AI chat transport & hooks
            └── ai-tools/      # Tool definitions for the AI assistant
```

### Crates

| Crate | Description |
|-------|-------------|
| `brewdio-core` | Brewing calculations, unit conversions, and ingredient/style data. Pure Rust, no I/O. |
| `brewdio-persistence` | SQLite-backed storage with Automerge CRDT sync. Compiles to both native (rusqlite) and WASM (sqlite-wasm-rs). |
| `brewdio-wasm` | wasm-bindgen bindings that expose core + persistence + SyncWorker to JavaScript/TypeScript. |
| `brewdio-server` | Axum WebSocket relay server. Receives changes from one client and broadcasts to all others via `tokio::sync::broadcast`. |
| `brewdio-tui` | Terminal UI built with ratatui. Full recipe/batch/settings management with vim-style navigation, live vitals, ingredient search, equipment profiles, and background sync. |

### Terminal UI

The `brewdio-tui/` crate is a full-featured terminal interface built with [ratatui](https://ratatui.rs/):

- **Vim-style navigation** — `j/k` movement, `n` to create, `d` to delete, `q` to go back
- **Three main tabs** — Recipes, Batches, Settings (switch with `Tab` or `1`/`2`/`3`)
- **Recipe editor** — Multi-panel layout with header, live vitals panel, and tabbed ingredient lists
- **Live vitals** — Real-time OG, FG, ABV, IBU, and SRM with color-coded range bars showing BJCP style guidance
- **Multi-step ingredient dialogs** — Fuzzy-searchable selection for fermentables, hops, and cultures with amount/unit/timing steps
- **Equipment profiles** — 40+ built-in profiles with efficiency %, searchable selector with confirmation prompt
- **Batch management** — Create batches from recipes, edit brew dates, independent batch notes
- **Trash & recovery** — Soft-delete recipes with `d`, toggle trash view with `r`, undelete with `u`
- **Notes editor** — Multiline modal editor for recipe and batch notes
- **Change history** — View Automerge CRDT change log with timestamps and actor IDs
- **Background sync** — Set `BREWDIO_SERVER_URL` to enable; status indicator in top-right corner (green = connected)
- **XDG data directory** — Database stored at `~/.local/share/brewdio/brewdio.db`

### Web UI

The `brewdio-webui/` directory is a React + TypeScript frontend that consumes `brewdio-wasm`:

- **[React 19](https://react.dev/)** with **[TanStack Router](https://tanstack.com/router)** (file-based routing)
- **[Tailwind CSS](https://tailwindcss.com/)** + **[shadcn/ui](https://ui.shadcn.com/)**
- **[Vite](https://vite.dev/)** for development and builds
- **[TanStack Query](https://tanstack.com/query)** for data fetching with automatic cache invalidation on sync
- Data stored in IndexedDB via the WASM SQLite layer

### Sync Architecture

```
┌──────────┐         ┌──────────────┐         ┌──────────┐
│  WebUI   │◄──WS──►│  brewdio-    │◄──WS──►│   TUI    │
│ (browser)│         │   server     │         │(terminal)│
└──────────┘         └──────────────┘         └──────────┘
     │                      │                       │
  IndexedDB             SQLite                   SQLite
```

- All sync protocol logic (Hello handshake, NewDoc exchange, SyncDoc messages) lives in Rust
- WebUI: `SyncWorker` WASM struct handles the protocol; JS handles only WebSocket transport (~100 lines)
- TUI: native `sync_worker.rs` runs as a tokio background task
- Server: relays changes between clients via broadcast channel; each connection maintains its own Automerge sessions

---

## Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [Bun](https://bun.sh/) 1.0+

### Development

```bash
# Run Rust tests (excludes WASM cdylib)
cargo test --workspace --exclude brewdio-wasm

# Build WASM package
cd brewdio-wasm && wasm-pack build --target web

# Run WASM integration tests
cd brewdio-wasm && bun install && bun test

# Start web UI dev server
cd brewdio-webui && bun install && bun dev

# Build and run the TUI
cargo run -p brewdio-tui

# Start the sync server (default port 3000)
cargo run -p brewdio-server
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BREWDIO_DB` | `brewdio.db` | Path to the SQLite database (server & TUI) |
| `BREWDIO_SERVER_URL` | — | WebSocket server URL for TUI sync (e.g. `ws://localhost:3000/ws`) |
| `PORT` | `3000` | Port for the sync server |

### Building for Production

```bash
# Build WASM
cd brewdio-wasm && wasm-pack build --target web

# Build web UI
cd brewdio-webui && bun install && bun run build

# Build server
cargo build -p brewdio-server --release
```

---

## Brewing Calculations

brewdio implements proven brewing formulas:

- **IBU (Tinseth Method)** — Accurate bitterness calculations accounting for alpha acids, boil time, and gravity
- **Original Gravity (OG)** — Based on fermentable extract and batch size
- **Final Gravity (FG)** — Estimated using yeast attenuation
- **ABV** — Standard formula: `(OG - FG) * 131.25`
- **SRM Color** — Morey equation for beer color prediction
- **Water Requirements** — Equipment-aware water calculations with grain absorption, boil-off, and dead space losses
- **Carbonation** — Priming sugar and forced carbonation calculations

---

## BeerJSON Compliance

brewdio uses [BeerJSON](https://beerjson.org/) as its native recipe format. This ensures:

- **Portability** — Share recipes with other BeerJSON-compatible software
- **Future-Proof** — Industry-standard format that will be supported for years
- **Complete Data** — Captures everything from ingredients to mash schedules
- **Extensible** — Easy to add new fields as needed

---

## Local-First Philosophy

- **Your Data, Your Device** — Everything is stored locally. No cloud servers required, no privacy concerns.
- **Offline by Default** — Works without an internet connection. Perfect for brew days in the garage.
- **CRDT Sync** — Optional multi-device sync via Automerge. Changes merge automatically, even after offline edits.
- **No Vendor Lock-in** — Export your recipes as standard BeerJSON files and use them anywhere.
- **Privacy First** — Zero tracking, zero telemetry, zero data collection.

---

## Contributing

brewdio is a work in progress! Contributions are welcome.

---

## License

This project is open source. Check the LICENSE file for details.

---

## Acknowledgments

- **[BeerJSON](https://beerjson.org/)** — For providing an excellent standard format
- **[Automerge](https://automerge.org/)** — For making local-first CRDT sync practical
- **Brewing Community** — For sharing formulas and best practices

---

**Happy Brewing!**
