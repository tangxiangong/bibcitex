# CLAUDE.md

This file provides guidance to Claude Code when working with code in this
repository.

## Project Overview

BibCiTeX is a cross-platform BibTeX citation management tool built with **Tauri
v2**, **React 19**, and **Deno**. It provides a desktop application for
researchers to manage, search, and quickly cite BibTeX references with global
shortcuts and cross-application paste functionality.

## Architecture

### Tech Stack

- **Backend**: Tauri v2 (Rust)
- **Frontend**: React 19 + React Router v7
- **Package Manager**: Deno
- **Styling**: TailwindCSS v4 + DaisyUI v5 (via Vite plugin, NO config files)
- **Math Rendering**: KaTeX

### Project Structure

```
bibcitex/
├── src/                    # React frontend
│   ├── main.tsx           # React app entry point
│   ├── App.tsx            # Main app component with routing
│   ├── app.css            # Global styles (TailwindCSS)
│   ├── layouts/           # Layout components
│   │   └── MainLayout.tsx # Main layout with drawer
│   ├── pages/             # Page components
│   │   ├── HomePage.tsx   # Bibliography management
│   │   ├── DetailPage.tsx # Reference detail page
│   │   └── HelperPage.tsx # Spotlight helper window
│   └── lib/
│       ├── components/    # React components
│       ├── context/       # React context (global state)
│       │   └── AppContext.tsx
│       ├── types.ts       # TypeScript types
│       └── tauri.ts       # Tauri IPC commands
├── src-tauri/             # Tauri backend (Rust)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs        # Entry point
│       ├── lib.rs         # App setup & plugins
│       ├── commands.rs    # Tauri IPC commands
│       ├── bib.rs         # BibTeX parsing
│       ├── setting.rs     # Settings management
│       ├── search.rs      # Reference search
│       ├── xpaste.rs      # Cross-app paste
│       └── error.rs       # Error handling
├── index.html             # HTML template
├── deno.json              # Deno configuration
├── package.json           # NPM compatibility
├── vite.config.ts         # Vite configuration
├── tsconfig.json          # TypeScript configuration
└── tailwind.config.js     # TailwindCSS configuration
```

## Development Commands

```bash
# Install dependencies
deno install

# Start development server
deno task dev

# Build for production
deno task build

# Type check
deno task check

# Format code
deno fmt

# Lint code
deno lint
```

## Key Features

### Global State Management (src/lib/context/AppContext.tsx)

React Context provides:

- `settings`: Application settings and bibliography list
- `currentReferences`: Currently loaded bibliography references
- `drawerOpen`: Boolean for reference details drawer visibility
- `drawerReference`: Currently selected reference for drawer display
- `currentBibName`: Name of currently loaded bibliography
- `updateSettings()`: Update settings
- `openDrawer()`: Open drawer with a reference
- `closeDrawer()`: Close drawer

Usage:

```tsx
import { useApp } from "@lib/context/AppContext";

function MyComponent() {
  const { settings, openDrawer, currentReferences } = useApp();
  // ...
}
```

### Tauri Commands (src-tauri/src/commands.rs)

- Settings: `load_settings`, `save_settings`, `add_bibliography`,
  `remove_bibliography`
- Bibliography: `load_bibliography`, `parse_bib_file`
- Search: `search_references`, `search_by_field`
- Clipboard: `copy_to_clipboard`, `paste_to_app`
- File: `select_bib_file`, `open_url`, `open_file`
- Update: `check_update`, `install_update`

### Tauri Plugins Used

- `tauri-plugin-clipboard-manager`: Clipboard access
- `tauri-plugin-dialog`: File dialogs
- `tauri-plugin-fs`: File system access
- `tauri-plugin-global-shortcut`: Global keyboard shortcuts (Cmd+Shift+K)
- `tauri-plugin-opener`: Open URLs and files
- `tauri-plugin-process`: Process management
- `tauri-plugin-updater`: Auto-update functionality

### Platform-Specific Features

- **macOS**: Global shortcuts, system tray, cross-app paste via AppleScript
- **Windows**: Global shortcuts, system tray, cross-app paste via Win32 API
- **Cross-platform**: Clipboard access, file dialogs, system notifications

## Supported Bibliography Types

- Article ✓
- Book ✓
- Thesis (MastersThesis, PhdThesis) ✓
- Booklet ✓
- InBook ✓
- InCollection ✓
- Misc ✓
- TechReport ✓
- InProceedings ✓
- Manual (WIP)
- Proceedings (WIP)
- Unpublished (WIP)

## Migration from Svelte to React

This project was migrated from Svelte 5 to React 19:

### Key Changes

1. **State Management**: Svelte stores → React Context API
2. **Routing**: SvelteKit → React Router v7
3. **Reactivity**: Svelte's `$state`, `$derived` → React's `useState`, `useMemo`
4. **Components**: `.svelte` files → `.tsx` files
5. **Build**: SvelteKit adapter → Standard Vite + React

### Component Patterns

**Svelte Pattern:**

```svelte
<script lang="ts">
  let count = $state(0);
  let doubled = $derived(count * 2);
</script>
```

**React Pattern:**

```tsx
import React, { useMemo, useState } from "react";

function Component() {
  const [count, setCount] = useState(0);
  const doubled = useMemo(() => count * 2, [count]);
}
```

### Tauri Integration

The Tauri backend and IPC layer remain unchanged. All Tauri commands in
`src/lib/tauri.ts` work exactly the same way in React as they did in Svelte.

## Development Notes

- Use path aliases: `@/`, `@lib/`, `@components/` for cleaner imports
- All Tauri commands are async and return Promises
- DaisyUI provides pre-built components that work with TailwindCSS
- KaTeX is used for rendering mathematical formulas in BibTeX fields

## TailwindCSS v4 Configuration

**IMPORTANT**: This project uses TailwindCSS v4 with the new Vite plugin
approach.

### ✅ Correct Setup (Current)

1. **Vite Plugin** - `vite.config.ts`:
   ```ts
   import tailwindcss from "@tailwindcss/vite";

   export default defineConfig({
     plugins: [react(), tailwindcss()],
   });
   ```

2. **CSS Import** - `src/app.css`:
   ```css
   @import "tailwindcss";
   @plugin "daisyui" {
     themes: winter --default, dracula --prefersdark;
   }
   ```

3. **Dependencies** - `package.json`:
   ```json
   {
     "dependencies": {
       "@tailwindcss/vite": "^4.1.18",
       "tailwindcss": "^4.1.18",
       "daisyui": "^5.5.14"
     }
   }
   ```

### ❌ DO NOT Create These Files

- `tailwind.config.js` - NOT used in v4
- `postcss.config.js` - NOT used in v4
- Any PostCSS configuration

TailwindCSS v4 uses CSS-based configuration via `@import` and `@plugin`
directives in your CSS file, processed by the Vite plugin. All configuration
(themes, custom utilities, plugins) should be defined in `src/app.css` using the
new v4 syntax.
