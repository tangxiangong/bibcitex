# BibCiTeX 0.8.2

## Citation formats and batch copying

- Add global citation settings for key, LaTeX, and Typst formats, with saved citation commands and multiple-citation output preferences.
- Support LaTeX `\cite`, natbib's `\citep` / `\citet`, and biblatex's `\autocite`, `\parencite`, `\textcite`, and `\footcite`; Typst supports `@key`, `#cite`, and prose citations.
- Add a Cite action to the main and tray windows for copying one or more references in result-list order. Helper direct paste and copy fallback after a failed paste use the same citation format.
- Choose merged or individual output for multiple citations; key output always uses commas. Copying BibTeX source preserves the original content.

After choosing a citation format in Settings, merged output for two references looks like this:

| Format | Example output |
| --- | --- |
| key | `Knuth1984,Lamport1994` |
| LaTeX / `\cite` | `\cite{Knuth1984,Lamport1994}` |
| Typst / `@key` | `@Knuth1984 @Lamport1994` |

## Multiple selection and tags

- Replace selection menus in the main and tray windows with compact horizontal floating toolbars for citations, batch tag editing, selecting all, clearing, and ending selection.
- Click the selection button again or press Esc to leave selection mode. Editors, popup menus, and IME composition retain priority for their own Escape handling.
- Show all assigned tags directly in reference details as capsules with color markers and a separate add control. Keep the current reference selected after tag changes and prevent stale asynchronous refreshes from replacing current results.

## Fixes

- Restore previous clipboard content after Helper paste where supported, without overwriting content copied during the paste.
- Fix tray windows immediately reopening when their icon is clicked again; a second click now closes the window.

<!-- platform:macos -->
## macOS

- Fix Shift / ⌘ clicks failing to enter multiple selection directly in the main and tray windows, and prevent stale updates from overriding selection.
- Show the Helper selection count only when two or more references are selected.
- Anchor tag editors beside list tag areas in the main window, tray, and Helper, or beside the detail add button; close stale editors when the selected reference changes.
<!-- /platform -->

<!-- platform:windows -->
## Windows

- Make Settings dropdowns narrower and consistent. Show only the selected citation command while retaining descriptions in the expanded list.
- Prevent double-clicking the tray icon from toggling the window repeatedly.
<!-- /platform -->
