# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BibCiTeX is a cross-platform BibTeX citation management tool built with Rust and Dioxus framework. It provides a desktop application for researchers to manage, search, and quickly cite BibTeX references with global shortcuts.

## Architecture

### Workspace Structure
- **Root package**: Main desktop application using Dioxus UI framework
- **bibcitex-core**: Core library containing BibTeX parsing, search, settings, and utilities
- **Components**: UI components in `src/components/` (bibliography, reference, helper, math)
- **Views**: Application views in `src/views/` (home, references, navigation, helper)
- **Assets**: Icons, styling, and app icons in `assets/`

### Key Technologies
- **Frontend**: Dioxus (React-like framework for Rust)
- **Styling**: TailwindCSS with DaisyUI components
- **BibTeX Parsing**: biblatex crate
- **Desktop Integration**: Global shortcuts (Cmd+K), file dialogs, clipboard access
- **Build Tool**: Just (Justfile for task automation)

## Development Commands

### Development Server
```bash
just serve          # Start dev server with CSS watch (parallel tasks)
# Or separately:
just dx-serve       # Start Dioxus dev server
just css-watch      # Watch and compile TailwindCSS
```

### CSS Management
```bash
just css            # Compile TailwindCSS once
just css-minify     # Compile and minify CSS for production
```

### Code Formatting
```bash
just fmt            # Format all code (Dioxus + Cargo + sort dependencies)
```

### Standard Rust Commands
```bash
cargo build         # Build the project
cargo run           # Run the application
cargo test          # Run tests
cargo clippy        # Lint with Clippy
```

## Key Application Features

### Global State Management
- `STATE`: Application settings and configuration
- `CURRENT_REF`: Currently loaded bibliography references
- `DRAWER_OPEN`/`DRAWER_REFERENCE`: UI drawer state for reference details

### Core Functionality
- **Bibliography Import**: Load and parse BibTeX files
- **Reference Search**: Real-time search through imported references
- **Global Shortcuts**: Cmd+K to open spotlight search window
- **Citation Copy**: Copy formatted citations to clipboard
- **Reference Details**: Drawer component for detailed reference information

### File Structure Notes
- Main entry point: `src/main.rs` (desktop launcher)
- App component: `src/lib.rs` with global shortcuts and routing
- Core logic: `bibcitex-core/src/` (bib parsing, search, settings, utils)
- Styling: Input CSS in `input.tailwind.css`, compiled to `assets/tailwind.css`

## Note (NEED TO IMPLEMENT)
In macOS,  `with_focused(false)` in `tao` does not work well, so we should call
system APIs to implement the **NO FOCUSED**  spotlight-style helper window,
such as [tauri-nspanel](https://github.com/ahkohd/tauri-nspanel),
which uses `tauri`, which is based on `tao`, saming as `Dioxus` we use here.
