# brewdio

> A **local-first** homebrewing app designed to be powerful and fun to use.

brewdio is a modern, privacy-focused brewing companion that puts you in control of your recipes and brew days. Everything runs locally—no accounts, no servers, no subscriptions. Just pure brewing.

---

## Features

### Recipe Management
- **BeerJSON Compliant** - Industry-standard recipe format ensures portability and future-proofing
- **Smart Calculations** - Automatic OG, FG, ABV, IBU, and color calculations using proven brewing formulas
- **Ingredient Database** - Comprehensive libraries of hops, fermentables, and yeast strains
- **Style Guidelines** - Built-in BJCP style ranges to help you hit your targets

### Batch Tracking
- **Brew Day Companion** - Read-only batch views perfect for following along during brewing
- **Equipment Profiles** - Customize loss calculations for your specific setup
- **Water Calculator** - Visual water requirement calculator with equipment-specific adjustments
- **Batch Notes** - Markdown-enabled notes for documenting your process and results

### Modern Brewing
- **Dry Hopping Support** - Full timing control for boil and fermentation hop additions
- **Equipment-Aware** - Water calculations adapt to your mash tun, kettle, and fermenter losses
- **Grain Absorption** - Accurate water calculations based on your grain bill

### User Experience
- **Local-First** - All data stored locally using SQLite (IndexedDB-backed in the browser, native on desktop)
- **Dark Mode** - Beautiful dark mode support for late-night brewing sessions
- **Retro Dials** - Playful retro-cockpit gauge displays for calculated values
- **Inline Editing** - Edit recipe values directly without cumbersome forms
- **JSON Editor** - Power users can edit raw BeerJSON for maximum control

---

## Architecture

brewdio is a Rust workspace with a shared core compiled to both WebAssembly and native targets:

```
brewdio/
├── brewdio-core/          # brewdio-core — brewing calculations & data
│   └── src/
│       ├── abv.rs         # ABV calculation
│       ├── og.rs          # Original gravity
│       ├── fg.rs          # Final gravity
│       ├── ibu.rs         # Tinseth IBU
│       ├── color.rs       # Morey SRM color
│       ├── water.rs       # Water calculator
│       ├── carbonation.rs # Carbonation calculator
│       ├── olfarve.rs     # Beer color rendering
│       ├── units.rs       # Unit conversions
│       └── data/          # Ingredient & style JSON databases
├── brewdio-persistence/   # brewdio-persistence — SQLite storage & CRDT sync
│   └── src/
│       ├── db.rs          # Recipe CRUD
│       ├── batch.rs       # Batch CRUD
│       ├── settings.rs    # User settings
│       ├── recipe.rs      # RecipeDocument (Automerge)
│       ├── sync.rs        # CRDT sync sessions
│       ├── connection_native.rs  # Native SQLite (rusqlite)
│       └── connection_wasm.rs    # WASM SQLite (sqlite-wasm-rs)
├── brewdio-wasm/          # WASM bindings + integration tests (bun test)
├── brewdio-tui/           # Terminal UI (ratatui)
├── brewdio-webui/         # brewdio-webui — React web frontend
│   └── src/
│       ├── routes/        # File-based routing (TanStack Router)
│       ├── components/    # UI components (shadcn/ui)
│       └── lib/           # Utilities & actions
├── Cargo.toml             # Workspace root
└── .github/workflows/     # CI (GitHub Actions)
```

### Crates

| Crate | Description |
|-------|-------------|
| `brewdio-core` | Brewing calculations, unit conversions, and ingredient/style data. Pure Rust, no I/O. |
| `brewdio-persistence` | SQLite-backed recipe, batch, and settings storage with Automerge CRDT sync. Compiles to both native (rusqlite) and WASM (sqlite-wasm-rs). |
| `brewdio-wasm` | wasm-bindgen bindings that expose core + persistence to JavaScript/TypeScript. |
| `brewdio-tui` | Terminal UI built with ratatui for recipe and batch management from the command line. |

### Web UI

The `brewdio-webui/` directory is a React + TypeScript frontend that consumes `brewdio-wasm`:

- **[React 19](https://react.dev/)** with **[TanStack Router](https://tanstack.com/router)**
- **[Tailwind CSS](https://tailwindcss.com/)** + **[shadcn/ui](https://ui.shadcn.com/)**
- **[Vite](https://vite.dev/)** for development and builds
- Data stored in IndexedDB via the WASM SQLite layer

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
wasm-pack build brewdio-wasm --target bundler

# Run WASM integration tests
cd brewdio-wasm && bun install && bun test

# Start web UI dev server
cd brewdio-webui && bun install && bun dev

# Build TUI
cargo build -p brewdio-tui
```

### Building for Production

```bash
# Build WASM
wasm-pack build brewdio-wasm --target bundler

# Build web UI
cd brewdio-webui && bun install && bun run build
```

---

## Brewing Calculations

brewdio implements proven brewing formulas:

- **IBU (Tinseth Method)** - Accurate bitterness calculations accounting for alpha acids, boil time, and gravity
- **Original Gravity (OG)** - Based on fermentable extract and batch size
- **Final Gravity (FG)** - Estimated using yeast attenuation
- **ABV** - Standard formula: `(OG - FG) * 131.25`
- **SRM Color** - Morey equation for beer color prediction
- **Water Requirements** - Equipment-aware water calculations with loss rates
- **Carbonation** - Priming sugar and forced carbonation calculations

---

## BeerJSON Compliance

brewdio uses [BeerJSON](https://beerjson.org/) as its native recipe format. This ensures:

- **Portability** - Share recipes with other BeerJSON-compatible software
- **Future-Proof** - Industry-standard format that will be supported for years
- **Complete Data** - Captures everything from ingredients to mash schedules
- **Extensible** - Easy to add new fields as needed

---

## Local-First Philosophy

- **Your Data, Your Device** - Everything is stored locally. No cloud servers, no sync issues, no privacy concerns.
- **Offline by Default** - Works without an internet connection. Perfect for brew days in the garage or basement.
- **CRDT Sync** - Optional peer-to-peer sync via Automerge for multi-device collaboration without a central server.
- **No Vendor Lock-in** - Export your recipes as standard BeerJSON files and use them anywhere.
- **Privacy First** - Zero tracking, zero telemetry, zero data collection.

---

## Contributing

brewdio is a work in progress! Contributions are welcome.

---

## License

This project is open source. Check the LICENSE file for details.

---

## Acknowledgments

- **[BeerJSON](https://beerjson.org/)** - For providing an excellent standard format
- **[Automerge](https://automerge.org/)** - For making local-first CRDT sync practical
- **Brewing Community** - For sharing formulas and best practices

---

**Happy Brewing!**
