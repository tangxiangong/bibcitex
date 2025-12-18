# CLAUDE.md

This file provides guidance to Claude Code when working with code in this
repository.

## Project Overview

BibCiTeX is a cross-platform BibTeX citation management tool built with **Tauri
v2**, **Svelte 5**, and **Deno**. It provides a desktop application for
researchers to manage, search, and quickly cite BibTeX references with global
shortcuts and cross-application paste functionality.

## Architecture

### Tech Stack

- **Backend**: Tauri v2 (Rust)
- **Frontend**: Svelte 5 + SvelteKit
- **Package Manager**: Deno
- **Styling**: TailwindCSS v4 + DaisyUI v5
- **Math Rendering**: KaTeX

### Project Structure

```
bibcitex/
├── src/                    # Svelte frontend
│   ├── app.html           # HTML template
│   ├── app.css            # Global styles (TailwindCSS)
│   ├── routes/            # SvelteKit routes
│   │   ├── +layout.svelte # Main layout
│   │   ├── +page.svelte   # Home page (Bibliography management)
│   │   ├── detail/        # Reference detail page
│   │   └── helper/        # Spotlight helper window
│   └── lib/
│       ├── components/    # Svelte components
│       ├── stores/        # Svelte stores (global state)
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
├── deno.json              # Deno configuration
├── package.json           # NPM compatibility
├── vite.config.ts         # Vite configuration
├── svelte.config.js       # SvelteKit configuration
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

### Global State Management (src/lib/stores/state.ts)

- `settings`: Application settings and bibliography list
- `currentReferences`: Currently loaded bibliography references
- `drawerOpen`: Boolean for reference details drawer visibility
- `drawerReference`: Currently selected reference for drawer display

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
