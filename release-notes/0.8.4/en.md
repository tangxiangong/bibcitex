# BibCiTeX 0.8.4

<!-- platform:macos,windows -->
## Tray and shortcut settings

- Add a tray-icon visibility switch and save the preference automatically.
- Record Helper shortcuts directly from the keyboard, apply changes immediately, and reset to the default shortcut inline. Keep the previous shortcut if replacement registration fails.
- Group language and theme controls under Appearance for a more compact settings page.

## Dependency updates

- Update shared-core and native-client dependencies.
<!-- /platform -->

<!-- platform:macos -->
## macOS

- Use system modifier symbols in the shortcut recorder.
- Update Sparkle and continue to ship one Universal installer and app archive for Apple Silicon and Intel.
<!-- /platform -->

<!-- platform:windows -->
## Windows

- Display Windows key names such as Win, Ctrl, Alt and Shift in shortcuts.
- Upgrade the Windows App SDK and align runtime installation and bootstrap versions to avoid selecting an older runtime.
<!-- /platform -->

## Release scope

- This release includes macOS and Windows. Linux release builds remain disabled pending native desktop acceptance.
