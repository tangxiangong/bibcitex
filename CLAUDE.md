# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Development Server
```bash
dx serve
```
Start the Dioxus development server for the desktop application.

### Building
```bash
cargo build
```
Build the project in debug mode.

```bash
cargo build --release
```
Build an optimized release version.

### Testing
```bash
cargo test
```
Run all tests in the workspace.

```bash
cargo test --package bibcitex-core
```
Run tests specifically for the core library.

### Benchmarking
```bash
cargo bench --package bibcitex-core
```
Run benchmarks for the core library (available in `bibcitex-core/benches/`).

### Linting
```bash
cargo clippy
```
Run Clippy linter with project-specific configuration from `clippy.toml`.

## High-Level Architecture

### Project Structure
This is a Rust workspace with two main crates:
- **Root crate (`BibCiTeX`)**: Desktop UI application using Dioxus framework
- **`bibcitex-core`**: Core library handling BibTeX parsing, search, and data structures

### UI Architecture (Dioxus Framework)
- **Router-based navigation**: Uses `dioxus::router` with routes defined in `src/route.rs`
- **Component hierarchy**: Main components in `src/components/`, views in `src/views/`
- **Global state management**: Uses Dioxus global signals for application state
- **CSS styling**: Located in `assets/styling/` directory with modular CSS files

### Key Components
- **Main App** (`src/lib.rs`): Entry point with global shortcut handler (Cmd+K)
- **Routing** (`src/route.rs`): Two main routes - Home and References with NavBar layout
- **Spotlight Window** (`src/views/helper.rs`): Cmd+K triggered overlay for quick citation access
- **Bibliography Management**: Components for importing and managing BibTeX files

### Core Library Architecture
- **BibTeX Parsing**: Handles parsing of BibTeX files into structured data
- **Search Functionality**: Provides search capabilities across references
- **Settings Management**: Configuration and user preferences
- **Error Handling**: Centralized error types and handling

### Cross-Platform Keyboard Automation
The application uses `enigo` crate for cross-platform clipboard and keyboard automation, enabling citation pasting into external applications.

### Global Shortcuts and Window Management
- **Global shortcut**: Cmd+K opens spotlight-style helper window
- **Window management**: Desktop-specific features using Dioxus desktop capabilities
- **Focus handling**: Automatic window closing when losing focus

### State Management
- **Global signals**: Used for application-wide state (settings, current references)
- **Local signals**: Component-specific state management
- **Debouncing**: Implemented for global shortcuts to prevent double-triggering

### Asset Management
- **Static assets**: Icons, logos, and CSS files served from `assets/` directory
- **Icon generation**: Python-based icon generation tool in `icon-gen/` directory

### Note
In macOS,  `no_focused` in `tao` does not work well, so we should call
system APIs to implement the **NO FOCUSED**  spotlight-style helper window,
such as [tauri-plugin-spotlight](https://github.com/zzzze/tauri-plugin-spotlight),
who uses `tauri`, which is based on `tao`, saming as `Dioxus` we use here.
However, the API used in `tauri-plugin-spotlight` is deprecated, so we
should use latest `objc2` to implement this feature.
