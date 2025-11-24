# 🍺 brewdio

> A **local-first** homebrewing app designed to be powerful and fun to use.

brewdio is a modern, privacy-focused brewing companion that puts you in control of your recipes and brew days. Everything runs locally in your browser—no accounts, no servers, no subscriptions. Just pure brewing.

---

## ✨ Features

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
- **Local-First** - All data stored in your browser using IndexedDB
- **Dark Mode** - Beautiful dark mode support for late-night brewing sessions
- **Retro Dials** - Playful retro-cockpit gauge displays for calculated values
- **Inline Editing** - Edit recipe values directly without cumbersome forms
- **JSON Editor** - Power users can edit raw BeerJSON for maximum control

---

## 🛠️ Tech Stack

brewdio is built with modern web technologies optimized for local-first applications:

### Core
- **[Bun](https://bun.sh/)** - All-in-one JavaScript runtime and toolkit
- **[React 19](https://react.dev/)** - UI library with cutting-edge features
- **[TypeScript](https://www.typescriptlang.org/)** - Type-safe development
- **[Vite](https://vite.dev/)** - Lightning-fast development and builds

### State & Data
- **[TanStack React DB](https://tanstack.com/db)** - Local-first reactive database built on Dexie/IndexedDB
- **[TanStack Store](https://tanstack.com/store)** - Reactive state management for calculations
- **[TanStack Form](https://tanstack.com/form)** - Type-safe form management

### Routing
- **[TanStack Router](https://tanstack.com/router)** - File-based routing with type-safe navigation

### UI & Styling
- **[Tailwind CSS](https://tailwindcss.com/)** - Utility-first CSS framework
- **[shadcn/ui](https://ui.shadcn.com/)** - High-quality, accessible components built on Radix UI
- **[Lucide Icons](https://lucide.dev/)** - Beautiful, consistent icon set
- **[next-themes](https://github.com/pacocoursey/next-themes)** - Seamless dark mode support

### Brewing
- **[BeerJSON](https://beerjson.org/)** - Industry-standard recipe format (v1.0.2)
- **Custom Calculation Engine** - Brewing math implementations (Tinseth IBU, color, etc.)

---

## 📁 Project Structure

```
brewdio/
├── src/
│   ├── routes/              # File-based routing (TanStack Router)
│   │   ├── index.tsx        # Home page
│   │   ├── recipes.$recipeId.tsx
│   │   ├── batches.$batchId_.overview.tsx
│   │   └── ...
│   ├── components/          # Reusable UI components
│   │   ├── ui/              # shadcn/ui components
│   │   ├── recipe-header.tsx
│   │   ├── add-hop-dialog.tsx
│   │   └── ...
│   ├── calculations/        # Brewing calculations
│   │   ├── ibu.ts           # Tinseth IBU calculation
│   │   ├── gravity.ts       # OG/FG calculations
│   │   ├── color.ts         # SRM color calculation
│   │   └── ...
│   ├── lib/                 # Utility functions
│   │   ├── actions/         # Data mutation helpers
│   │   ├── calculate.ts     # Calculation engine
│   │   └── ...
│   ├── contexts/            # React contexts
│   │   └── recipe-edit-context.tsx
│   ├── data/                # Static data
│   │   ├── hops.json        # Hop varieties
│   │   ├── styles.json      # BJCP styles
│   │   └── ...
│   ├── db.ts                # Database schema & collections
│   └── main.tsx             # App entry point
├── public/                  # Static assets
└── package.json
```

---

## 🚀 Getting Started

### Prerequisites
- [Bun](https://bun.sh/) 1.0+

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/brewdio.git
cd brewdio

# Install dependencies
bun install

# Start development server
bun dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

### Building for Production

```bash
# Build optimized production bundle
bun run build

# Preview production build locally
bun preview
```

The built files will be in the `dist/` directory, ready to deploy to any static hosting service.

---

## 🌐 Local-First Philosophy

brewdio embraces the **local-first** software philosophy:

- **Your Data, Your Device** - Everything is stored in your browser's IndexedDB. No cloud servers, no sync issues, no privacy concerns.
- **Offline by Default** - Works without an internet connection. Perfect for brew days in the garage or basement.
- **No Vendor Lock-in** - Export your recipes as standard BeerJSON files and use them anywhere.
- **Privacy First** - Zero tracking, zero telemetry, zero data collection. What you brew is your business.
- **Instant Response** - No network latency means instant UI updates and calculations.

### Data Storage

All data is stored using [Dexie.js](https://dexie.org/) (a wrapper around IndexedDB):
- **Recipes** - Your beer recipes in BeerJSON format
- **Batches** - Brew sessions with snapshots of recipes and equipment
- **Equipment** - Equipment profiles with loss rates and volumes
- **Settings** - User preferences and defaults

---

## 🍺 BeerJSON Compliance

brewdio uses [BeerJSON](https://beerjson.org/) as its native recipe format. This ensures:

- **Portability** - Share recipes with other BeerJSON-compatible software
- **Future-Proof** - Industry-standard format that will be supported for years
- **Complete Data** - Captures everything from ingredients to mash schedules
- **Extensible** - Easy to add new fields as needed

You can view and edit the raw BeerJSON using the built-in JSON editor, or use the friendly UI for common tasks.

---

## 🧪 Brewing Calculations

brewdio implements proven brewing formulas:

- **IBU (Tinseth Method)** - Accurate bitterness calculations accounting for alpha acids, boil time, and gravity
- **Original Gravity (OG)** - Based on fermentable extract and batch size
- **Final Gravity (FG)** - Estimated using yeast attenuation
- **ABV** - Standard formula: `(OG - FG) * 131.25`
- **SRM Color** - Morey equation for beer color prediction
- **Water Requirements** - Equipment-aware water calculations with loss rates

---

## 🎨 Design Philosophy

brewdio aims to be:

- **Fun** - Retro gauge dials, smooth animations, and playful UI elements
- **Powerful** - Advanced features for experienced brewers without cluttering the interface
- **Fast** - Local-first architecture means instant response times
- **Beautiful** - Dark mode support and attention to visual details
- **Accessible** - Built with semantic HTML and ARIA attributes

---

## 🤝 Contributing

brewdio is a work in progress! Contributions are welcome.

### Development

```bash
# Install dependencies
bun install

# Run dev server with hot reload
bun dev

# Type checking
bun run build  # Also runs tsc -b

# Linting
bun lint
```

### Code Style
- TypeScript strict mode enabled
- Functional components with hooks
- File-based routing with TanStack Router
- Component colocation (keep related files together)

---

## 📝 License

This project is open source. Check the LICENSE file for details.

---

## 🙏 Acknowledgments

- **BeerJSON** - For providing an excellent standard format
- **Brewing Community** - For sharing formulas and best practices
- **TanStack** - For amazing local-first tooling
- **shadcn** - For beautiful, accessible components

---

**Happy Brewing! 🍻**
