# Release note platform rules

Keep reviewed UTF-8 Markdown in `<version>/zh-Hans.md` and `<version>/en.md`.
Use the same platform scopes in both translations.

- Unmarked content is common and appears on every platform. Existing notes remain compatible.
- `<!-- platform:all -->` explicitly starts a common block.
- `<!-- platform:macos -->`, `<!-- platform:windows -->`, and `<!-- platform:linux -->`
  start platform-only blocks. Names are lowercase and case-sensitive.
- Comma-separated names match any listed platform, e.g. `<!-- platform:macos,windows -->`.
  Do not combine `all` with other names.
- Every block ends with `<!-- /platform -->`. Blocks cannot nest.
- Each marker occupies its own line. Surrounding whitespace is allowed.
  Unknown names, empty selectors, malformed markers, nesting, and unmatched markers
  fail generation, including inside excluded blocks. These markers are reserved even
  inside code fences; do not include literal examples of them in release announcements.
- Keep platform-specific headings inside their blocks to avoid empty headings.
  Content order and Markdown formatting are preserved; markers are removed.
- Language selection is independent of platform selection. Each locale must produce
  nonempty content for all three platforms; include a common version heading or summary.

Example source:

```markdown
# 0.8.0

- Improve bibliography parsing for everyone.

<!-- platform:macos -->
## macOS
- Fix quick-citation focus restoration.
<!-- /platform -->

<!-- platform:windows,linux -->
- Improve installation on Windows and Linux.
<!-- /platform -->
```

Preview locally without packaging or publishing:

```sh
cargo run --locked -p xtask -- release-notes release-notes/0.8.0 target/release-notes-preview
```

The generator validates both translations before writing output. It emits
`notes-{platform}-{locale}.md` for macOS, Windows, and Linux. The compatibility
files `notes-{locale}.md` contain **macOS-filtered** content for existing Sparkle
links. `notes-windows.md` combines Windows-filtered translations using the existing
locale markers for Velopack. `release-body.md` contains both languages and all
platforms, with visible platform labels and no condition markers, for the GitHub
Release editor. GitHub does not select content by the visitor's operating system.
The publisher preserves the maintainer's Release body; copy the generated body
into the editor when creating the release. The body is not an uploaded asset.
GitHub/COS publication includes the other generated files in the release checksums. Windows packaging runs the same generator before packing.
Linux assets are ready for consumption; the current Linux client has no announcement
viewer. This does not retrofit or overwrite previously published releases.
