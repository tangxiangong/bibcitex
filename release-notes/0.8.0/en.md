# BibCiTeX 0.8.0

## More flexible search

- Ordinary search matches multiple terms across default fields such as title and author. Improve Unicode, accent, punctuation, prefix and substring matching, with limited spelling tolerance for eligible words and normal matches ranked first.
- Add advanced search with fields, consecutive phrases, year comparisons and ranges, reference types, tags, AND / OR / NOT, and parentheses. Conditions can also be composed in the main or tray window's condition editor.
- Keep separate drafts for ordinary and advanced search, and improve ranking and caching for repeated searches.

## Using advanced search

Enable advanced search in the main or tray window (off by default and remembered separately). Helper uses advanced search by default. Ordinary mode does not interpret syntax such as `author:` or `OR`.

| Goal                                    | Example                              |
| --------------------------------------- | ------------------------------------ |
| Match both terms                        | `graph network`                      |
| Match a consecutive phrase              | `"machine learning"`                 |
| Author and minimum year                 | `author:Knuth year:>=2020`           |
| Title term, excluding surveys           | `title:graph -survey`                |
| Either term, with an author restriction | `(graph OR network) author:Smith`    |
| Inclusive year range                    | `year:2020..2024`                    |
| Articles or books                       | `type:(article OR book)`             |
| Both tags                               | `tag:reading tag:math`               |
| Either tag, excluding read items        | `tag:(math OR algorithms) -tag:read` |
| Full citation key                       | `key:Smith2024`                      |

- Spaces mean AND. Boolean operators must be uppercase. Precedence is parentheses → `NOT` / `-` → AND → OR. `-survey` equals `NOT survey`; do not put a space after the minus sign.
- Use English field names and place the value immediately after the colon: `author:Knuth`, not `author: Knuth`. `title:(graph neural)` restricts both terms to the title.
- Other fields include `editor`, `journal`, `booktitle`, `abstract`, `note`, `publisher`, `school`, `institution`, `organization`, `doi`, and `isbn`. `intitle` aliases `title`; `citekey` aliases `key`. Citation keys, DOIs and ISBNs match complete identifiers.
- Years support `>`, `>=`, `<`, `<=`, `=`, and ranges. `year:2020..` means 2020 onward; `year:..2024` means up to and including 2024.
- Use ASCII double quotes for phrases or tags containing spaces, such as `tag:"machine learning"`. Tags match complete names; unknown tags are not created automatically.
- Complete missing values, quotes or parentheses before searching. Invalid or incomplete queries disable actions on stale results. Quote literal operators, for example `"AND"` or `"title:graph"`.

### Helper quick filters

1. Type `@` for reference types, `$` for fields, or `#` for existing tags at a whitespace boundary at the end of the input.
2. Filter candidate names in the small input, navigate with arrow keys, and confirm with Enter or a click. Types and tags immediately become condition capsules.
3. Fields have a separate optional value input: choose year, enter `2026`, and press Enter to create `year:2026`. Leave it empty and press Enter, Tab or Esc to use that field as the scope for subsequent keywords instead.
4. Continue typing keywords or adding conditions. At the start of the trailing input with no selection, Backspace removes the preceding capsule. Esc cancels candidates.

IME composition takes priority. Pasted text, email addresses and symbols inside words such as `C#` do not trigger candidates. See the [complete advanced search guide](https://github.com/tangxiangong/bibcitex/blob/v0.8.0/README.md#advanced-search-guide).

## Tags and multiple selection

- Add colored tags with creation, renaming, recoloring, merging, deletion and broken-association recovery. Tags are shared across libraries and retain stable identities when libraries move or are relocated, without modifying `.bib` files.
- Select multiple references in the main and tray windows to add, remove, or create and assign tags in one batch, with full/partial association states and validation before writing the whole batch.
- In Helper, a normal click pastes immediately. ⌘-click (Ctrl-click on Windows) toggles selection; Shift-click or Shift+arrows extends it; ⌘/Ctrl+A selects all current results. Enter pastes selected citation keys in list order, joined with commas.
- Show selected counts and the paste destination application when identifiable. Searching, filtering or switching libraries clears selection to prevent stale actions.
- Back up both `setting.json` and `search-metadata.json` from the configuration directory to preserve tags. Ordinary BibTeX import/export does not carry tags.

Batch copying from the main/tray windows and LaTeX or Typst citation-format presets are not yet available.

<!-- platform:macos -->
## macOS

- Fix selection highlight flashes and scroll content insets; improve narrow-window layouts, toolbars and detail panes.
- Improve tray sheet retention, search focus restoration and selection state, plus Helper scrolling, footer layout and accessible paste activation.
- Fix attachment-path parsing for colons and escaped delimiters.
<!-- /platform -->

<!-- platform:windows -->
## Windows

- Align search controls and toolbar buttons in the main and tray windows; improve candidate popup placement, scrolling and keyboard selection.
- Adjust Helper's footer so hints do not overlap results, while keeping the idle panel compact.
- Fix cancellation of stale asynchronous paste requests so obsolete requests do not continue executing.
<!-- /platform -->
