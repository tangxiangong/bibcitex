# BibCiTeX 0.7.1

This update refines the native desktop experience, fixes cross-application paste from quick citation, and improves update downloads and Windows installation.

## New and improved

- Native settings windows on macOS and Windows bring language, appearance, update channel, and automatic-update preferences together. On macOS, open settings with `⌘,`.
- The tray browser now offers floating reference details, hidden by default to preserve list space. Click a citation key to copy it, and press `Esc` to dismiss the details.
- Refined main-window split panes, list selection, and scrollbar behavior, with better row alignment and selection indicators in quick citation.
- macOS now prefers system icons and features refreshed tray artwork. Windows gains improvements to title bars, light and dark themes, settings layout, and icon presentation.
- Improved formula rendering: fixed inconsistent sizing between initial rendering and cached content on macOS, and refined inline formula sizing, spacing, and baseline alignment on Windows.
- About windows on both platforms now display license information.

## Fixes

- Fixed a possible macOS crash during cross-application paste from quick citation and improved focus handoff before pasting. Opening quick citation keeps the main window hidden.
- On both platforms, failed cross-application paste now automatically copies the citation to the clipboard for manual pasting.
- Fixed copy feedback affecting unrelated controls on macOS and older feedback replacing newer confirmations during repeated copying.

## Updates and installation

- In-app updates now prefer Tencent Cloud COS and fall back to GitHub if update-feed requests or package downloads fail, while retaining download integrity checks.
- Windows installs missing .NET, Visual C++, and Windows App Runtime dependencies as needed and reuses compatible installed versions. Internet access is required if dependencies are missing during installation or first launch.
- Minimum system requirements are now macOS 13 Ventura and Windows 10 version 1809 (build 17763). Apple Silicon and Intel Macs, plus Windows x64 and ARM64 PCs, remain supported.

Cross-application paste on macOS requires Accessibility permission for BibCiTeX in System Settings.
