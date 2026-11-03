# BibCiTeX 0.8.3

<!-- platform:macos,windows -->
## Nested tag filters

- Build nested AND, OR and NOT tag groups in the main and tray windows, and combine them with keyword, field and reference-type filters.
- Edit conditions in a compact tag popover, with changes applied together when confirmed. Tag merges refresh selected IDs; missing tags and incomplete groups do not silently broaden results.

## Updates

- Add automatic update-check preferences and defer update operations while application work is active.
<!-- /platform -->

<!-- platform:macos -->
## macOS

- Ship one Universal installer and app archive for Apple Silicon and Intel. Existing architecture-specific update feeds receive the Universal app.
- Keep the main window hidden after Command-H when opening the tray panel.
<!-- /platform -->

<!-- platform:windows -->
## Windows

- Verify signed update metadata before trusting package hashes, while retaining COS delivery and GitHub fallback.
- Add persisted automatic-check scheduling, installation on exit, and clearer update status; report download timeouts and reject stale cached packages.
- Keep Helper candidate colors synchronized with theme changes and accommodate scaled text without clipping.
<!-- /platform -->

## Release scope

- This release includes macOS and Windows. Linux release builds remain disabled pending native desktop acceptance.
