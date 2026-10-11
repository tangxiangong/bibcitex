<div align="center">
  <img src="public/transparent_logo.png" width="120" alt="BibCiTeX">

  <p>English | <a href="README-zh.md">简体中文</a></p>

  <p>
    A quick citation tool for BibTeX bibliographies
  </p>

  <p>
    <a href="https://github.com/tangxiangong/bibcitex/releases">
      <img src="https://img.shields.io/github/v/release/tangxiangong/bibcitex?style=for-the-badge&logo=github&color=blue" alt="GitHub Release">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/blob/main/LICENSE-MIT">
      <img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue?style=for-the-badge" alt="License">
    </a>
  </p>

  <p>
    <strong>Quick Downloads</strong>
  </p>

  <p>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.7.3/BibCiTeX-0.7.3-macos-arm64.dmg">
      <img src="https://img.shields.io/badge/macOS-Apple Silicon-000000?style=for-the-badge&logo=apple&logoColor=white" alt="macOS Apple Silicon">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.7.3/BibCiTeX-0.7.3-macos-x86_64.dmg">
      <img src="https://img.shields.io/badge/macOS-Intel-000000?style=for-the-badge&logo=apple&logoColor=white" alt="macOS Intel">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.7.3/BibCiTeX-0.7.3-windows-arm64.exe">
      <img src="https://img.shields.io/badge/Windows-ARM64-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Windows ARM64">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.7.3/BibCiTeX-0.7.3-windows-x64.exe">
      <img src="https://img.shields.io/badge/Windows-x86__64-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Windows x86_64">
    </a>
  </p>
</div>

## Introduction

**BibCiTeX** is a quick citation tool for **BibTeX** bibliographies, powered by a shared **Rust** core.

It supports macOS and Windows, with bibliography search, citation copying, and cross-application pasting.

What's new in v0.7.3: [English release notes](release-notes/0.7.3/en.md) · [Chinese release notes](release-notes/0.7.3/zh-Hans.md).

### Core Features

- **One-click copying**: Copy citations with ease.
- **Cross-application pasting**: Paste citations directly into your workflow.

## Installation

### Supported Systems

| Platform | Supported OS Versions | Architectures |
| --- | --- | --- |
| macOS | macOS 13 (Ventura) or later | Apple Silicon (ARM64), Intel (x86_64) |
| Windows | Windows 10 version 1809 (OS build 17763) or later, including Windows 11 | x64, ARM64 |

### Downloads

Download the latest installer for your platform and architecture from the [**Releases page**](https://github.com/tangxiangong/bibcitex/releases).

### macOS Installation

You can also install through the project's [Homebrew tap](https://github.com/tangxiangong/homebrew-tap), which supports Apple Silicon and Intel Macs running macOS 13 or later:

```bash
brew install --cask tangxiangong/tap/bibcitex
```

The app supports built-in updates. To update through Homebrew, run `brew update`, followed by `brew upgrade --cask --greedy bibcitex`.

If macOS reports that `BibCiTeX` is damaged, open Terminal and run:

```bash
sudo xattr -dr com.apple.quarantine /Applications/BibCiTeX.app
```

> **Tip**: This message is caused by macOS security restrictions. The command above removes the app's quarantine attribute so you can open it.

## Interface Preview

The animations below show earlier versions of the app.

<div align="center">

### Core Features

|                          Add `.bib` Files                          |                              Bibliography List                               |                           Smart Search                           |
| :----------------------------------------------------------------: | :--------------------------------------------------------------------------: | :--------------------------------------------------------------: |
| [<img src="assets/add_bib.gif" width="120">](./assets/add_bib.gif) | [<img src="assets/show_details.gif" width="120">](./assets/show_details.gif) | [<img src="assets/search.gif" width="120">](./assets/search.gif) |
|                   *Quickly import BibTeX files*                    |                           *View reference details*                           |                   *Filter results as you type*                   |

|                         Details Sidebar                          |                       External Links                       |                        Copy Citations                        |
| :--------------------------------------------------------------: | :--------------------------------------------------------: | :----------------------------------------------------------: |
| [<img src="assets/drawer.gif" width="120">](./assets/drawer.gif) | [<img src="assets/url.gif" width="120">](./assets/url.gif) | [<img src="assets/copy.gif" width="120">](./assets/copy.gif) |
|                  *View details in the sidebar*                   |            *Quickly access external resources*             |          *Copy formatted citations with one click*           |

### Featured Functionality

<div style="margin: 20px 0;">
  <h4>Cross-Application Pasting</h4>
  <a href="assets/cross_paste.gif">
    <img src="assets/cross_paste.gif" alt="Cross-application paste demo">
  </a>
  <p><em>Paste citations across applications for seamless integration into your workflow.</em></p>
</div>

</div>

## Advanced Search Guide

macOS, Windows and Linux share the Rust advanced search engine, explicitly using **chumsky** for lexing and parsing. Enable **Advanced Search** separately in the main and tray windows: both switches default to off, persist independently, and retain separate ordinary/advanced drafts. The helper defaults to advanced search. Ordinary mode does not interpret `author:`, `OR` or other advanced syntax.

### Quick start

Type a query directly or open the condition editor to add conditions, AND/OR groups and exclusions. Both entry methods use the same query rules.

| Find | Query |
| --- | --- |
| Both keywords, including across default fields | `graph network` |
| A continuous phrase | `"machine learning"` |
| An author and a year constraint | `author:Knuth year:>=2020` |
| A title term, excluding another term | `title:graph -survey` |
| Either keyword, with the same author constraint | `(graph OR network) author:Smith` |
| Articles or books | `type:(article OR book)` |
| Years 2020–2024, inclusive | `year:2020..2024` |
| Both tags | `tag:unread tag:math` |
| Either tag, excluding read references | `tag:(math OR algorithms) -tag:read` |
| A complete citation key | `key:Smith2024` |

Spaces imply AND; explicit `AND` works too. Only uppercase `AND`, `OR` and `NOT` are Boolean operators; lowercase forms are ordinary text. Precedence is parentheses → `NOT` / `-` → AND → OR. `graph OR network author:Smith` means `graph OR (network AND author:Smith)`; use the parenthesized example above to constrain both branches. `NOT survey` equals `-survey`, with no space after the minus. Exclusion-only queries are valid. The selected library and external filters constrain the entire expression.

### Fields, phrases and identifiers

Field names are case-insensitive. Values or opening parentheses must immediately follow the colon: `author:Knuth` is valid; `author: Knuth` is not. Use the English syntax names below and ASCII double quotes for multiword phrases.

| Field | Searches |
| --- | --- |
| `title` (alias `intitle`) | Title |
| `author`, `editor` | Authors and editors; each condition may match any person in the list |
| `journal` | Abbreviated or full journal name |
| `booktitle` | Containing book, collection or conference proceedings title |
| `abstract`, `note` | Abstract and notes |
| `publisher`, `school`, `institution`, `organization` | Publishers, school, institution and organizations |
| `key` (alias `citekey`) | Complete citation key, case-insensitive but preserving accents and punctuation |
| `doi` | Complete DOI, ignoring case and common DOI URL / `doi:` prefixes |
| `isbn` | Complete ISBN, ignoring whitespace and ASCII hyphens; ISBN-10 and ISBN-13 are not automatically equivalent |
| `year` | Year, comparison or range |
| `type` | Reference type |
| `tag` | Complete tag name |

Unqualified terms search citation keys, titles, authors, journals, years and notes within the selected default scope. Extended metadata such as abstracts requires an explicit field. `title:graph neural` limits only graph to the title; `title:(graph neural)` limits both terms. Explicit inner fields override inherited scope, as in `title:(graph OR author:Smith)`. `author:Smith author:Jones` can match two different authors; `author:"Smith Jones"` must occur within one author entry.

Unquoted text supports case, accent and common punctuation tolerance, plus prefix/substring matching. Double quotes require a continuous phrase: case and repeated whitespace are normalized, but accents, punctuation, mathematical symbols and word boundaries are preserved. Phrases cannot span fields or separate author entries. `"machine learning"` matches neither `machine deep learning` nor `machine learnings`. Quoted DOI, ISBN, key, year, type and tag values retain their field-specific matching rules.

Only positive unqualified ordinary terms are eligible for spelling supplementation. With fewer than 20 normal results, words of 5–64 Latin letters in titles, authors and journals may allow one spelling difference. Explicit fields, phrases, identifiers, years, types, tags and negative conditions do not use it; a matching predicate also appearing negatively is ineligible throughout the query. Normal hits rank ahead of supplemented hits, whose corrected terms are indicated.

### Years and types

| Syntax | Meaning |
| --- | --- |
| `year:2026` or `year:=2026` | Exactly 2026 |
| `year:>2020` or `after:2020` | After 2020, excluding 2020 |
| `year:<2024` or `before:2024` | Before 2024, excluding 2024 |
| `year:>=2020` or `year:2020..` | 2020 or later |
| `year:<=2024` or `year:..2024` | 2024 or earlier |
| `year:2020..2024` | 2020 through 2024, inclusive |

Years are integers from 1 to 9999. Ranges cannot be reversed, and comparison operators cannot be separated from their values by whitespace. `before` / `after` accept one year and refer to the reference's publication year. A reference without a year fails a positive year condition but can satisfy its negation. Bare `2020..2024` is ordinary text.

Type values are case-insensitive: `article`, `book`, `thesis`, `mastersthesis`, `phdthesis`, `booklet`, `inbook`, `incollection`, `inproceedings`, `misc`, `techreport`. `thesis` includes general, master's and doctoral theses. `type:article type:book` requires both types and returns no results; use OR for alternatives. Bibliography support for Manual, Proceedings and Unpublished remains incomplete.

### Tags

Use tag management to create, rename, delete or explicitly merge tags, and reference details to assign or remove tags for individual references. Tags live in separate local metadata and never modify `.bib` files. Back up the application's configuration-directory `search-metadata.json` when migrating libraries; ordinary BibTeX import/export does not include tags.

Tag matching uses the complete name, normalizes whitespace and ignores case, but preserves accents and punctuation. Quote names containing spaces, for example `tag:"machine learning"`. An unknown tag matches no references; `-tag:unknown` excludes none. Unknown names produce a warning and do not create tags automatically.

Tags selected through candidates or the condition editor retain stable IDs and follow renames. Handwritten `tag:old-name` resolves by name and does not follow renames. A deleted selected tag makes its condition invalid until removed or reselected; it never silently broadens the search. Ambiguous reference changes remain pending for manual reassociation in tag management.

### Helper: floating candidates and capsules

1. At a whitespace boundary at the end of the input, type `@` (type), `$` (field) or `#` (existing tag). A small inline input and floating candidate list appear immediately.
2. The first small input **only filters candidate names**. For example, type `$` then `author` to narrow the field list, or `@` then `book` to narrow types. Chinese UI accepts Chinese names plus corresponding English names and syntax identifiers; English UI uses English names and identifiers only. Custom tags keep their original names.
3. Choose with Up/Down and Enter, or click. Types and tags become capsules immediately and require no value; the caret returns after the capsules.
4. Selecting a field opens a **separate optional value input**. Select `year`, enter `2026`, then Enter to create `year:2026` and restore the previous keyword scope. Empty Enter, Tab or Esc skips the value and keeps the field as the keyword scope. For example, select author, skip the value and type Knuth in the trailing input to search authors. Explicit fields, types and tags override this default scope.
5. Confirmation updates results using valid conditions. Continue typing keywords or adding capsules. With the trailing caret at the beginning and no selection, Backspace removes the preceding capsule and its query condition; repeated presses remove more. Text, selections and IME composition retain native deletion behavior.

Esc cancels the candidate picker and restores the previous draft. The floating list anchors beside the inline input, scrolls after six visible rows and does not occupy the results region. It has no Cancel / Tag management footer buttons. During Chinese composition, the input method owns arrows, Enter and Esc; the Enter that commits Chinese text does not also select a condition or paste a result. Backspace inside the candidate filter or optional value input edits only that input.

Triggers require actual typing: paste, programmatic changes, email addresses, `C#`, formulas and symbols inside existing words do not open candidates. Without keywords or effective conditions, the helper remains compact and idle; a field-scope capsule alone does not load the whole library.

### Literal symbols and errors

`graph-based`, `C++` and `C#` are ordinary text. Use `"AND"`, `"-survey"`, `"title:graph"` or `"f(x)"` for literal content. Inside quotes, `\"` represents a double quote and `\\` a backslash. Outside quotes, `\(`, `\)`, `\"`, `\\`, `\:` and an initial `\-` escape structural characters; other backslash sequences are preserved. Unknown prefixes such as `abc:def` and URLs remain ordinary text, so a typo such as `titel:graph` is not corrected to `title:graph`. Google web operators such as `site:` and `filetype:` are not supported.

A trailing `author:`, unclosed quote/parenthesis or trailing `OR` is incomplete. `author: Knuth`, empty quotes/parentheses and reversed year ranges are invalid. Definite errors take priority over incomplete suffixes. Incomplete or invalid queries do not execute, and old results cannot be copied/pasted until the query is completed or corrected.

## TODO

See [TODO.md](./TODO.md) for the detailed roadmap.

- [x] **Search improvements**: Multiple keywords, fuzzy matching, relevance ranking, and performance optimization.
- [x] **Advanced search**: Fields, exact phrases, Boolean groups, exclusions, year ranges and a visual condition editor, powered by chumsky.
- [x] **Tags**: Single-reference assignment, catalog management, stable identities and tag filtering.
- [ ] **Bulk tags**: Add and remove tags across multiple selected references.
- [x] **Helper filter commands**: Floating `@` / `$` / `#` candidates, inline name filtering, optional field values and removable capsules on all three platforms.
- [ ] **Multiple selection**: Select multiple references and perform bulk operations in the main window and helper.
- [ ] **Copy and paste formats**: Bare citation keys, LaTeX and Typst citation presets, and combined citations for multiple references.

## Third-Party Code Attribution

### [crates/xpaste](./crates/xpaste)

- **Source**: [EcoPasteHub/EcoPaste](https://github.com/EcoPasteHub/EcoPaste)
- **Author**: EcoPasteHub
- **License**: [Apache 2.0](https://github.com/EcoPasteHub/EcoPaste/blob/master/LICENSE)
- **Usage**: Cross-application paste functionality.
- **Copyright**:
  ```
  Copyright (c) EcoPasteHub
  ```
- **Modifications**:
  - macOS: Replaced deprecated `objc` and `cocoa` APIs with `objc2` APIs.
  - Windows: Replaced deprecated `winapi` APIs with `windows-sys` APIs.

---

> **Details**: See [**NOTICE**](./NOTICE) for complete attribution information.

## License

This project is dual-licensed under either of the following, at your option:

* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

### Contributions

Unless you explicitly state otherwise, any contribution you intentionally submit for inclusion in this project, as defined in the Apache-2.0 license, will be dual-licensed as above, without any additional terms or conditions.
