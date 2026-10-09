# BibCiTeX 0.7.3

This release is available for macOS and Windows. No Linux version is included.

## Common updates

- Mark libraries whose files are missing as unavailable while preserving the saved default library, avoiding a file-reading error at startup. Quick citation prompts you to choose an available library.
- Show the selected reference's title in the details header and keep BibTeX expanded for easier access.
- Place citation-key and BibTeX copy buttons beside their fields. Open URLs, DOIs, and file paths in the main window directly from their values.
- Unify rounded selection styling across the main window, tray, and quick citation, and refine the placement of the paste-citation-key and switch-library controls.
- Move the reference-details toggle to the right edge of the toolbar.
- Support platform-specific release notes: the app shows common updates and updates for its platform, while GitHub shows the complete announcement.

<!-- platform:macos -->
## macOS

- Refine glass materials, rounded corners, shadows, and fade transitions in quick citation and the tray, retaining compatible materials on older macOS versions.
- Fix missing icons in library action menus.
- Adjust the minimum main-window width to prevent layout loops and crashes when restoring a narrow window with reference details visible.
<!-- /platform -->

<!-- platform:windows -->
## Windows

- Fix text-pointer and insertion-caret visibility in the main-window and quick-citation search fields.
- Unify search-field styling, improve focus visibility over translucent backgrounds, and support high-contrast mode.
- Refine tray detail panels and toolbars, and adjust scrollbar visibility in the main window and tray.
<!-- /platform -->
