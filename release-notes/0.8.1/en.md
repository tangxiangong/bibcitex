# BibCiTeX 0.8.1

## Formula compatibility

- Correctly interpret doubled backslashes before multi-letter commands in simple mathematical expressions, such as `\\alpha` and `\\ell`, from bibliography exports.
- Preserve line breaks and row-bearing TeX constructs such as matrices, along with the original BibTeX source used for export.
- Keep advanced-search match positions aligned with the normalized formula text.

<!-- platform:macos -->
## macOS

- Fix narrow formula glyphs such as `\ell` disappearing during SVG rendering, while preserving formula size and alignment.
- Display the original formula text if image rendering fails.
- Keep scrollbars hidden in the main workbench, tray window, and sidebar/detail popovers, including while scrolling or hovering. Scrolling remains available.
<!-- /platform -->
