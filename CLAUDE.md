# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project Overview

BibCiTeX is a cross-platform BibTeX citation management desktop app built with
**Tauri v2** (Rust backend), **SolidJS** (frontend), and **Bun** (package
manager). Researchers use it to manage, search, and quickly cite BibTeX
references with global shortcuts (Cmd+Shift+K) and cross-application paste.

## Development Commands

```bash
bun install           # Install dependencies
bun run tauri:dev     # Start Tauri dev (frontend + backend)
bun run dev           # Start Vite dev server only (frontend)
bun run tauri:build   # Production build (Tauri)
bun run build         # Production build (Vite only)
bun run check         # TypeScript type check
```

## Architecture

### Tech Stack

- **Backend**: Tauri v2 (Rust) — `src-tauri/`
- **Frontend**: SolidJS + @solidjs/router — `src/`
- **Build**: Vite 8 with `vite-plugin-solid` + `@tailwindcss/vite`
- **Styling**: TailwindCSS v4 + DaisyUI v5
- **Math**: KaTeX for LaTeX rendering in BibTeX fields

### Frontend Architecture

**Entry**: `src/main.tsx` → `render()` from `solid-js/web`

**Routing** (`src/App.tsx`): Uses `@solidjs/router` with nested routes:
- `/` → `MainLayout` → `HomePage` (bibliography management)
- `/detail` → `MainLayout` → `DetailPage` (reference list + search)
- `/helper` → `HelperPage` (standalone spotlight search window)

**State** (`src/context/AppContext.tsx`): SolidJS Context with signals. All
context values are signal getters (functions):

```tsx
const { settings, currentReferences, drawerOpen } = useApp();
// Access: settings(), currentReferences(), drawerOpen()
// Functions like openDrawer, closeDrawer are plain functions (not signals)
```

**Tauri IPC** (`src/tauri.ts`): Framework-agnostic async wrappers around
`invoke()`. These are plain functions, not tied to any UI framework.

**Component patterns**: Each bibliography entry type (Article, Book, Thesis,
etc.) has three component variants:
- `src/components/reference/` — Card display with copy/detail actions
- `src/components/drawer/` — Full detail view in side drawer
- `src/components/helper/` — Compact display for spotlight search

Selector components (`ReferenceSelector`, `ReferenceDrawer`, `HelperSelector`)
dispatch to the correct variant via `switch` on `entry.type_`.

### Backend Architecture (`src-tauri/`)

- `lib.rs` — App setup: plugins, global shortcut, system tray, menus
- `commands.rs` — All 18 IPC command handlers
- `core/bib.rs` — `Reference` struct and BibTeX parsing (via `biblatex` crate)
- `core/setting.rs` — Settings persistence (`~/.config/BibCiTeX/setting.json`)
- `core/search.rs` — Parallel search with Rayon for large datasets
- `xpaste/` — Platform-specific cross-app paste (macOS: objc2, Windows: Win32)

### Two-Window Architecture

The app has two windows:
1. **Main window** (1200×800) — Full bibliography management UI
2. **Helper window** (spotlight) — Floating search panel triggered by global
   shortcut, dynamically resizes based on content. macOS uses NSPanel (via
   `tauri-nspanel`); Windows uses always-on-top undecorated window.

## SolidJS Conventions

This project uses SolidJS (not React). Key differences:

- **Signals**: `createSignal`, `createMemo`, `createEffect` — values are getter
  functions that must be called: `count()` not `count`
- **Control flow**: `<Show>`, `<For>`, `<Switch>`/`<Match>` components instead
  of ternaries and `.map()`
- **Props**: Do NOT destructure reactive props — use `props.x`. Destructuring
  in leaf components receiving static data from `<For>` is safe.
- **Refs**: Plain `let ref: HTMLElement | undefined` instead of `useRef`
- **Lifecycle**: `onMount`, `onCleanup` instead of `useEffect`
- **Router**: `<A href="/">` (not `<Link to="/">`), `props.children` (not
  `<Outlet/>`)
- **HTML**: Use `class` (not `className`), `innerHTML` (not
  `dangerouslySetInnerHTML`), `for` (not `htmlFor`)

## TailwindCSS v4 Configuration

**IMPORTANT**: TailwindCSS v4 uses the Vite plugin approach. There are NO
config files.

- Config lives in `src/app.css` via `@import "tailwindcss"` and
  `@plugin "daisyui"`
- **DO NOT** create `tailwind.config.js`, `postcss.config.js`, or any PostCSS
  config
- The Vite plugin (`@tailwindcss/vite`) handles everything
- DaisyUI themes: `winter` (default light), `dracula` (dark)

## Path Aliases

- `@/` → `src/`
- `@components/` → `src/components/`
- `@context/` → `src/context/`

Configured in both `vite.config.ts` (resolve.alias) and `tsconfig.json` (paths).

## Tauri Commands

All frontend-callable commands (defined in `src-tauri/src/commands.rs`):

- **Settings**: `load_settings`, `save_settings`, `add_bibliography`,
  `remove_bibliography`
- **Bibliography**: `load_bibliography`, `parse_bib_file`
- **Search**: `search_references`, `search_by_field`
- **Clipboard**: `copy_to_clipboard`, `paste_to_app`
- **File**: `select_bib_file`, `open_url`, `open_file`
- **Update**: `check_update`, `install_update`
- **Helper**: `resize_helper_window`, `open_helper_window`, `get_helper_bib`,
  `set_helper_bib`

## Supported Bibliography Types

Article, Book, Thesis (Masters/PhD), Booklet, InBook, InCollection, Misc,
TechReport, InProceedings — all supported. Manual, Proceedings, Unpublished are
WIP.
