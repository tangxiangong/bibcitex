# WARP.md && CLAUDE.md

This file provides guidance to WARP (warp.dev) and Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BibCiTeX is a cross-platform BibTeX citation management tool built with Rust and Dioxus framework. It provides a desktop application for researchers to manage, search, and quickly cite BibTeX references with global shortcuts and cross-application paste functionality.

## Architecture

### Workspace Structure
- **Main application**: Desktop app using Dioxus UI framework (`src/main.rs`, `src/lib.rs`)
- **bibcitex-core**: Core library for BibTeX parsing, search, settings, and utilities
- **nspanel**: macOS NSPanel integration for system-level unfocused windows (WIP, adapted from tauri-nspanel)
- **updater**: GitHub-based app update functionality (adapted from Tauri updater plugin)
- **xpaste**: Cross-application paste functionality (adapted from EcoPaste)

### Key Technologies
- **Frontend**: Dioxus v0.7.0-rc.0 (React-like framework for Rust)
- **Styling**: TailwindCSS v4.1.11 with DaisyUI v5.0.50
- **BibTeX Parsing**: biblatex v0.10
- **Desktop Integration**: Global shortcuts, system tray, clipboard, file dialogs
- **Build Tools**: Just (Justfile), Bun (CSS), dx CLI

### UI Architecture
- **Router**: Two-route system (Home `/`, References `/detail`)
- **Global State**: Signals for settings, current references, drawer state
- **Components**: Modular UI in `src/components/` (bibliography, reference, helper, math, updater)
- **Views**: Page-level components in `src/views/` (home, references, nav, helper, updater)

## Development Commands

### Development Server
```bash
just serve          # Start dev server with CSS watch (runs dx-serve + css-watch in parallel)
# Or separately:
dx serve           # Start Dioxus dev server only
just css-watch     # Watch and compile TailwindCSS only
```

### CSS Management
```bash
just css            # Compile TailwindCSS once
just css-minify     # Compile and minify CSS for production
bunx tailwindcss -i ./input.tailwind.css -o ./assets/tailwind.css --watch  # Direct CSS watch
```

### Code Formatting & Build
```bash
just fmt            # Format all code (dx fmt + cargo sort + cargo fmt)
cargo build         # Build the project
cargo run           # Run the application
cargo test          # Run tests
cargo clippy        # Lint with Clippy
```

### Icon Generation
```bash
just icon           # Generate app icons using Python script in icon-gen/
just desktop-icon   # Generate cross-platform desktop icons
```

## Key Application Features

### Global State Management (src/lib.rs:32-35)
- `STATE`: Application settings and configuration (`Setting::load`)
- `CURRENT_REF`: Currently loaded bibliography references (`Option<Vec<Reference>>`)
- `DRAWER_OPEN`: Boolean for reference details drawer visibility
- `DRAWER_REFERENCE`: Currently selected reference for drawer display

### Core Functionality
- **Bibliography Management**: Import/parse BibTeX files, support for multiple citation types
- **Real-time Search**: Filter references by multiple fields
- **Global Shortcuts**: Cmd+Shift+K (macOS) / Super+Shift+K to open spotlight-style helper window
- **System Integration**: Menu bar app, tray icon, cross-application paste
- **Auto-updater**: GitHub-based update checking and installation

### Platform-Specific Features
- **macOS**: NSPanel integration (WIP), proper activation policy handling, unfocused windows
- **Windows**: Windows-specific build configurations and icons
- **Cross-platform**: Clipboard access, file dialogs, system notifications

## File Structure Notes
- **Entry point**: `src/main.rs` (desktop launcher with platform-specific window setup)
- **App root**: `src/lib.rs` (main App component with routing, global shortcuts, tray)
- **Routing**: `src/route.rs` (simple two-route structure with NavBar layout)
- **Core logic**: `crates/bibcitex-core/src/` (bib parsing, search, settings, error handling, filters, utils)
- **CSS pipeline**: `input.tailwind.css` → `assets/tailwind.css`
- **Assets**: Static assets in `assets/` (icons, logo, CSS), built icons in `icons/`

## crates/ Directory

### bibcitex-core (crates/bibcitex-core/src/)
Core business logic including:
- `bib.rs`: BibTeX parsing and Reference struct with detailed fields
- `setting.rs`: Configuration management and bibliography tracking
- `search.rs`: Reference search and filtering
- `filter.rs`: Advanced filtering capabilities
- `error.rs`: Error handling
- `utils.rs`: Utility functions

### Platform-Specific Crates
- **nspanel**: macOS NSPanel wrapper for Dioxus (adapted from tauri-nspanel v2.1) **WIP**
- **updater**: GitHub release-based updater (adapted from Tauri updater plugin) **OK**
- **xpaste**: Cross-application paste functionality (adapted from EcoPaste) **OK**

## Platform Integration Notes
- **macOS unfocused windows**: Current implementation uses tao's `with_focused(false)` which doesn't work properly. The nspanel crate is being developed to use native NSPanel APIs for true system-level unfocused windows.
- **Tray integration**: Uses Dioxus desktop tray functionality with custom menu items
- **Global shortcuts**: Integrated with Dioxus desktop's global shortcut system

## Configuration Files
- **Dioxus config**: `Dioxus.toml` (bundle settings, app metadata)
- **Package metadata**: `Cargo.toml` (workspace configuration, dependencies)
- **Build tasks**: `Justfile` (development commands)
- **CSS pipeline**: `input.tailwind.css` → `assets/tailwind.css` (via Bun/TailwindCSS)

## Common Development Patterns

### Adding New BibTeX Fields
1. Update `Reference` struct in `crates/bibcitex-core/src/bib.rs`
2. Modify the `From<&biblatex::Entry>` implementation
3. Update UI components in `src/components/reference.rs` for display
4. Add search/filter support in `crates/bibcitex-core/src/search.rs`

### Working with Global State
- Use `STATE.read()` to access settings
- Use `CURRENT_REF.write()` to update loaded references
- Use `DRAWER_OPEN` and `DRAWER_REFERENCE` for UI state management

### CSS Development
- Always run `just css-watch` during development for live CSS updates
- Use DaisyUI components where possible for consistency
- CSS is processed through TailwindCSS v4.1.11 pipeline
