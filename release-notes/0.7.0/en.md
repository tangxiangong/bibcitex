# BibCiTeX 0.7.0

## Native desktop experience

- Native SwiftUI / AppKit on macOS and WinUI 3 on Windows, backed by a shared Rust bibliography core.
- A native library workbench, compact quick-citation window, and independent tray browser support search, filtering, copying, and cross-application paste.
- Rename, pin, and remove libraries. Removing a library does not delete the original `.bib` file.
- Navigate references with arrow keys from the search field, retain sidebar and detail-pane visibility, and enjoy clearer empty states and pane backgrounds.
- Simplified Chinese and English interfaces, with improved menu, filter-label, and reference-list refresh when switching languages.
- Refreshed app and tray icons, with layered Icon Composer artwork on macOS and matching Windows assets.

## Updates and installation

- Check GitHub Releases for updates on Stable, Beta, and Alpha channels.
- Sparkle on macOS and Velopack on Windows provide localized release notes, download integrity checks, and optional automatic updates.
- Switching channels never automatically downgrades the app. Changing channels or automatic-update preferences cancels stale background downloads.
- Supports Apple Silicon and Intel Macs running macOS 14 Sonoma or later, and x64 and ARM64 PCs running Windows 10 version 2004 (build 19041) or later.
- DMG installers for macOS; EXE and MSI installers for Windows.

Install this version manually when switching from an older version. The previous updater, settings, and installer are not migrated automatically. Keep your original `.bib` files, then add your libraries again and review settings after installation. Subsequent versions use the new in-app updater. Cross-application paste on macOS requires Accessibility permission.
