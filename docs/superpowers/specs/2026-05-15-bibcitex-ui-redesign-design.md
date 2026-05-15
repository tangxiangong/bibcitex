# BibCiTeX UI Redesign Design

Date: 2026-05-15

Status: design approved for review. This document changes no implementation code.

## Context

BibCiTeX is a Tauri v2 desktop app with a SolidJS frontend. The current product already supports:

- Managing multiple `.bib` bibliographies.
- Loading and parsing BibTeX references.
- Browsing references by type.
- Searching all fields or specific fields such as author, title, journal, and year.
- Viewing reference details, abstract, note, and raw BibTeX.
- Copying cite keys.
- Opening DOI, URL, and attached files.
- Opening a global shortcut helper window with `Cmd/Win + Shift + K`.
- Pasting a selected cite key back into the previous app.
- Checking and installing updates.

The redesign should preserve those capabilities while making the product feel like a fast writing companion rather than a decorative library demo.

## Product Model

BibCiTeX should be redesigned as an Instant Citation System.

The app has two primary surfaces:

- Main window: a three-pane bibliography workbench for managing libraries, browsing references, and inspecting details.
- Helper window: a command palette for the fastest writing flow: summon, search, select, paste.

The high-frequency path is:

1. User presses `Cmd/Win + Shift + K`.
2. User types a query.
3. User selects a result with arrow keys or pointer.
4. User presses `Enter`.
5. BibCiTeX pastes the selected cite key into the previous app and closes the helper.

The main window remains important, but its job is browsing, verification, and setup. The helper is the fastest citation path.

## Theme System

Use the Catppuccin theme for daisyUI directly.

Required theme model:

- Light mode: use Catppuccin `latte` globally.
- Dark mode: use Catppuccin `mocha` globally.
- Main window, helper window, detail panes, drawers, modals, update banner, empty states, and error states all follow the same current app theme.
- Do not lock the helper to dark mode.
- Do not hand-copy Catppuccin color values into component code.
- Do not mix Catppuccin themes with the current `winter` / `dracula` defaults.

Implementation should use the Catppuccin daisyUI theme integration as the source of truth and then rely on daisyUI semantic classes such as:

- `bg-base-100`, `bg-base-200`, `bg-base-300`
- `text-base-content`
- `btn-primary`, `btn-ghost`, `btn-error`
- `badge`, `badge-info`, `badge-success`, `badge-warning`, `badge-error`
- `alert-info`, `alert-success`, `alert-warning`, `alert-error`
- `border-base-300`

Custom CSS may define layout, spacing, sizing, and motion, but not a competing color system.

## Main Window

The main window uses a three-pane layout.

### Left Pane: Bibliographies

The left pane lists all configured bibliographies.

Each library row should show:

- Library name.
- Optional description.
- File path, abbreviated but inspectable.
- Updated time.
- Reference count when available.
- Active library state.
- Default helper library marker when applicable.
- Load/error state if the file is missing or cannot be parsed.

Primary actions:

- Add `.bib`.
- Select library.
- Open source file.
- Remove library.
- Set or recognize the active helper library.

The left pane must be collapsible. When hidden, the center pane expands.

### Center Pane: Reference List

The center pane shows references for the selected bibliography.

It should include:

- Sticky search input.
- Type filter.
- Field filter.
- Result count.
- Compact reference rows.
- Keyboard-selectable row state.
- Empty, loading, no-results, and error states.

Reference rows should be dense but confidence-building. Each row should include:

- Reference type badge.
- Title.
- Cite key as a monospace token.
- Author or editor preview.
- Year.
- Venue when available: journal, book title, publisher, school, institution, or organization.
- Availability hints for DOI, URL, file/PDF, abstract, note, or raw BibTeX when useful.

The row should not become a large card. It should scan more like a research list or command result.

### Right Pane: Selected Reference Detail

The right pane shows details for the selected reference.

It should include:

- Type and cite key.
- Title.
- Authors or editors.
- Bibliographic metadata appropriate to the type.
- DOI, URL, and file/PDF actions.
- Abstract.
- Note.
- Raw BibTeX source in a monospace block.
- Copy cite key action.

The right pane must be collapsible. When hidden, the center pane expands.

On desktop, this persistent detail pane replaces the always-overlay detail drawer for ordinary browsing. On narrow screens, the right detail surface can fall back to an overlay drawer or sheet.

### Collapsible Pane Behavior

Both side panes must be independently collapsible.

Supported states:

- Left visible, right visible: full three-pane browsing.
- Left hidden, right visible: reference list plus detail focus.
- Left visible, right hidden: library plus list browsing.
- Left hidden, right hidden: full-width reference list.

Pane controls should be icon buttons with labels/tooltips, stable hit targets, and visible focus states.

Pane widths should have stable min/max values. Collapsing or expanding a pane must not change row heights or resize unrelated controls in a visually jarring way.

Pane collapsed state must persist within the current running app session. Cross-session persistence is a follow-up implementation decision unless the implementation plan adds a UI preference store.

## Helper Window

The helper is a command palette.

Core behavior:

- Open with `Cmd/Win + Shift + K`.
- Focus the search input immediately.
- Show active bibliography context compactly.
- Search as the user types.
- Arrow keys move selection.
- `Enter` pastes the selected cite key into the previous app.
- `Escape` closes the helper.
- Pointer selection is supported.

### Search Model

The helper uses one search input.

Do not add upfront type or field filter controls to the helper. Advanced filtering belongs in the main window.

The helper search should cover title, author, cite key, journal/venue, year, and note according to existing search capabilities. If implementation keeps backend command boundaries, the UI should still present a single simple search surface.

### Result Rows

Helper rows should be compact and keyboard-first.

Each row should show:

- Title.
- Cite key as a monospace token.
- Author preview.
- Reference type.
- Year.
- One or two availability hints such as DOI/PDF when present.

The active row must be visually distinct in both `latte` and `mocha`.

### Helper States

No bibliography configured:

- Show a short explanation.
- Provide an action to open the main window and add a `.bib`.

Library selection mode:

- Show a compact searchable list of bibliographies.
- Sort by default/recent update.
- Show load state and errors inline.

Empty query:

- Show a prompt or recent context.
- Do not show a blank panel.

No results:

- Show the active query.
- Show search tips.
- Keep the active bibliography visible.

Paste success:

- Show a brief local confirmation, then close the helper.

Paste failure:

- Keep the helper open.
- Keep the cite key visible.
- Provide a copy fallback.
- Explain recovery when the cause is likely accessibility or focus permissions.

Resize failure:

- Must not block searching.
- Treat as nonfatal.

## Semantic Color Encoding

Different reference types and metadata properties must be visually distinguishable.

This distinction should use daisyUI semantic classes under Catppuccin themes, not hardcoded color values.

### Reference Type Mapping

Every supported reference type should have a stable visual identity.

Use:

- Type badge.
- Left row accent.
- Subtle selected/tinted state where contrast allows.

Supported type groups:

- Article.
- Book.
- Thesis, including MastersThesis and PhdThesis.
- TechReport.
- Misc.
- Booklet.
- InBook.
- InCollection.
- InProceedings.
- Unknown or unimplemented types.

The implementation plan should define a documented mapping from type to semantic variant. For example, one type may use `badge-info`, another `badge-success`, another `badge-warning`, but the exact class pair must be verified in both `latte` and `mocha`.

Avoid turning the list into a noisy rainbow wall. Prefer compact badges, left borders, and subtle surfaces over large saturated blocks.

### Metadata Property Mapping

Reference properties should also have stable visual treatment.

Required property groups:

- Title.
- Author and editor.
- Year and month.
- Journal, book title, publisher, school, institution, organization, and venue-like fields.
- DOI, URL, and file/PDF.
- Cite key.
- Abstract and note.
- Pages, volume, number, edition, series, ISBN, and other secondary metadata.
- Raw BibTeX source.

Examples of treatment:

- Title: strongest text treatment, never hidden behind low-contrast color.
- Author/editor: person or contributor chip.
- Year/date: compact numeric chip.
- DOI/URL/file: link/action chip.
- Cite key: monospace token.
- Abstract/note: readable content block.
- BibTeX: monospace source block.

The center list and right detail pane must reuse the same property identity so users learn one visual language.

### Light and Dark Contrast

Light and dark modes need separate semantic color treatments.

Do not assume one badge or chip variant works in both `latte` and `mocha`.

Rules:

- Maintain separate accepted class combinations for `latte` and `mocha`.
- Normal text and chip labels target WCAG AA contrast.
- Selected rows, focus rings, borders, and left accents must remain visible in both themes.
- If a `soft` badge or tinted background fails contrast in either theme, use `outline`, a neutral surface, stronger text, or a clearer border.
- Color can support meaning but must not be the only indicator.
- Every colored type or metadata chip also needs text, an SVG icon, a field label, or a stable layout position.

## Icon System

All UI icons must be SVG or vector icons.

Emoji are forbidden for structural UI.

This applies to:

- Navigation icons.
- Pane collapse/expand icons.
- Copy, details, open DOI, open URL, open file/PDF actions.
- Search icons.
- Update icons.
- Error, warning, success, ready, loading, and empty-state icons.
- Reference type icons.
- Metadata property icons such as title, author, year, journal, DOI, URL, file, abstract, note, and BibTeX.
- Keyboard helper hints if they include icons.

The current UI has emoji-like metadata markers in some reference/helper components. Those should be replaced during implementation.

Use one consistent SVG icon style across the product. Icon-only buttons must include:

- Accessible label.
- Tooltip where useful.
- At least 44 px interactive target.
- Visible hover, active, disabled, and focus states.

## Layout and Interaction Rules

Use desktop utility density:

- Compact rows.
- Stable row heights.
- 8 px radius as the normal surface radius unless daisyUI component constraints require otherwise.
- 8/12/16 px spacing rhythm.
- No large marketing hero inside the app.
- No nested cards inside cards.
- No decorative blobs, bokeh, shine overlays, or animated gradient text.

Motion:

- Use 150-220 ms transitions for pane collapse, helper selection, toast/banner changes, and drawer/sheet fallback.
- Animations must use transform or opacity when possible.
- Respect reduced motion.
- Do not animate layout in ways that make rows jump.

Keyboard behavior:

- Helper supports input focus, arrow selection, Enter paste, Escape close.
- Main reference list supports keyboard row selection.
- Focus order should follow visual order.
- Route or pane changes should keep focus predictable.

## Error and Empty States

Every error state must include a recovery action.

Examples:

- Load failed: show library path, retry, open source file, or remove broken library.
- Parse failed: identify the file and provide retry/open-file.
- Open DOI/URL/file failed: show what could not be opened and keep the action available.
- Paste failed: show copy fallback and recovery guidance.
- Update failed: show retry and close.

Empty states:

- No bibliographies: show add `.bib` action.
- No selected bibliography: explain that a library must be selected.
- No references loaded: provide a path back to libraries.
- No search results: show search tips and reset/filter adjustment action.
- No selected reference: show a quiet detail-pane placeholder.

## Accessibility Requirements

Required:

- Text contrast should meet WCAG AA in both `latte` and `mocha`.
- Field chips and type badges must be checked in both themes.
- Color must not be the only semantic cue.
- Icon-only controls need accessible labels.
- Focus states must be visible.
- Keyboard navigation must reach all functionality.
- Touch/pointer targets should be at least 44 px even on desktop where practical.
- Toasts and banners should not steal focus.
- Destructive actions need confirmation or clear undo/recovery.

## Implementation Boundaries

This design does not require backend feature changes for the first implementation pass.

The redesign should work with existing concepts:

- `Setting` and configured bibliographies.
- `Reference` data from the Rust backend.
- Existing load/search/copy/paste/open commands.
- Existing helper window command path.
- Existing update banner command/event path.

Implementation may refactor frontend components to avoid duplicated reference layouts, but the redesign should stay scoped to UI structure, theming, interaction states, and presentational organization.

## Acceptance Checklist

Design acceptance:

- App uses Catppuccin daisyUI themes.
- Light mode is globally `latte`.
- Dark mode is globally `mocha`.
- Main window is a three-pane workbench.
- Left pane lists bibliographies.
- Center pane lists references for the selected bibliography.
- Right pane shows selected reference details.
- Left and right panes are independently collapsible.
- Narrow screens use drawer/sheet fallback for side panes.
- Helper is a command palette.
- Helper `Enter` pastes the selected cite key.
- Helper keeps single-search-input simplicity.
- Reference types have stable visual identities.
- Metadata properties have stable visual treatments.
- Type and property color mappings are contrast-checked separately in `latte` and `mocha`.
- All structural icons are SVG/vector icons.
- No emoji are used as UI icons.
- No decorative blobs, shine overlays, or animated gradient text remain in core surfaces.
- Errors include recovery actions.

Later technical validation after implementation:

- Run `bun run check`.
- Verify app visually in `latte` and `mocha`.
- Verify pane collapse/expand states.
- Verify helper keyboard flow.
- Verify paste success and paste failure fallback.
- Verify no horizontal overflow at narrow widths.
- Verify SVG-only icon usage in UI components.
