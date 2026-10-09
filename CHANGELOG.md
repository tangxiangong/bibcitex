# Changelog

All notable changes to this project will be documented in this file.

## [0.7.3] - 2026-10-09

### Features and interface

- Mark libraries with missing files as unavailable, preserve the saved default, and guide quick citation back to library selection.
- Use reference titles as detail headers, keep BibTeX expanded, and place copy actions beside their fields.
- Make URL, DOI, and main-window file values directly clickable.
- Unify rounded selections across the workbench, tray, and helper; refine helper controls and move the details toggle to the toolbar's right edge.
- Refine macOS glass panels, shadows, corners, and transitions, with materials compatible with older macOS versions.
- Refine Windows tray panels, toolbars, and scrollbar visibility.

### Bug fixes

- Restore macOS library action menu icons and prevent layout loops when restoring narrow main windows with reference details visible.
- Restore Windows search pointers and insertion carets, and improve focus visibility and high-contrast styling.

### Updates and release scope

- Filter in-app release notes to common and matching platform updates; generate a complete bilingual GitHub announcement with platform labels.
- Publish macOS and Windows only. Temporarily disable Linux Release CI, including builds, signing, artifact upload, and publication, until the client is ready; retain ordinary Linux Check CI.
- Update the Homebrew tap after successful publication.

Release notes: [简体中文](release-notes/0.7.3/zh-Hans.md) · [English](release-notes/0.7.3/en.md) · [GitHub announcement](release-notes/0.7.3/release-body.md).

## [0.7.2] - 2026-10-07

### Bug fixes

- Keep the macOS main window hidden when opening quick citation with the global shortcut, including after a failed paste.
- Include WinUI resources in Windows publish output.

### Installation

- Document installation and upgrades through the project Homebrew tap.

Release notes: [简体中文](release-notes/0.7.2/zh-Hans.md) · [English](release-notes/0.7.2/en.md).

## [0.7.1] - 2026-10-07

### 🚀 Features

- *(settings)* Add native settings windows on macOS and Windows for language, appearance, update channels, and automatic updates; support `⌘,` on macOS.
- *(tray)* Add floating reference details, clickable citation keys for copying, and Escape dismissal.
- *(about)* Display license information on both platforms.
- *(updates)* Prefer Tencent Cloud COS for update feeds and downloads, with GitHub fallback and download integrity checks.

### 🐛 Bug Fixes

- *(helper)* Fix a possible macOS cross-application paste crash and improve focus handoff while keeping the main window hidden when opening quick citation.
- *(helper)* Automatically copy citations to the clipboard when cross-application paste fails on either platform.
- *(macos)* Isolate copy feedback between controls and prevent stale confirmations from replacing newer feedback.
- *(formulas)* Fix inconsistent initial and cached formula sizing on macOS; improve inline formula sizing, spacing, and baseline alignment on Windows.

### 🎨 Styling

- Refine split panes, list selection, scrollbar behavior, and quick-citation row alignment and selection indicators.
- Prefer system icons on macOS and refresh tray artwork; improve Windows title bars, themes, settings layout, and icons.

### Updates and installation

- Install missing Windows .NET, Visual C++, and Windows App Runtime dependencies as needed, reusing compatible installed versions. Missing dependencies require internet access during installation or first launch.
- Lower minimum system requirements to macOS 13 Ventura and Windows 10 version 1809 (build 17763), retaining support for Apple Silicon, Intel, Windows x64, and Windows ARM64.

Release notes: [简体中文](release-notes/0.7.1/zh-Hans.md) · [English](release-notes/0.7.1/en.md).

## [0.5.1] - 2025-10-09

### Dependencies

- Update dependencies

## [0.5.0] - 2025-08-23

### 🚀 Features

- *(filter)* Add new reference type filtering functions
- *(references)* Add filter components and expand reference types

### 🐛 Bug Fixes

- *(macos)* Exit process after spawning updater

## [0.4.1] - 2025-08-22

### 🐛 Bug Fixes

- *(bibliography)* Clear helper bib when deleting bibliography

## [0.4.0] - 2025-08-21

### 🚀 Features

- *(updater)* Add new updater crate with platform-specific support
- *(update)* Add update checking and installation functionality
- *(updater)* Implement cross-platform updater with GitHub integration

### 🚜 Refactor

- *(crates)* Restructure code into separate crates for better modularity
- Remove linux support and cleanup code

## [0.3.0] - 2025-08-17

### 🚀 Features

- *(components)* Add support for Booklet, InBook and InCollection entry types

## [0.2.0] - 2025-08-16

### 🚀 Features

- Change window close behavior to hide instead of close
- Add custom menu, about dialog and tray helper menu
- Add dark mode support with color variants and hover states
- *(macos)* Add custom event handler for window activation policy

### 🐛 Bug Fixes

- *(windows)* Prevent window flickering on startup by delaying visibility

### 📚 Documentation

- Add installation instructions for macOS users

## [0.1.0] - 2025-08-16

### 🚀 Features

- **Core Functionality**
  - BibTeX parsing and bibliography management
  - Reference search with parallel processing support
  - Copy to clipboard functionality for cite keys
  - Support for multiple reference types (Article, Book, Thesis, Misc, TechReport, InProceedings)
  - Math formula rendering with KaTeX support

- **User Interface**
  - Modern UI with Tailwind CSS and DaisyUI themes
  - Spotlight-style citation helper tool with global hotkey
  - Reference filtering and field-specific search
  - Drawer component for reference details
  - Navigation bar and routing system

- **Cross-Platform Support**
  - macOS, Windows, and Linux compatibility
  - Platform-specific window management and paste functionality
  - Keyboard automation and focus handling

- **Performance**
  - Parallel processing for large bibliographies
  - Automatic optimization based on file size thresholds

### 🐛 Bug Fixes

- Fixed cite key search normalization and BibTeX field parsing
- Improved cross-platform window focus and paste reliability
- Enhanced search functionality and filter behavior
- Corrected UI layout and styling issues

### 💼 Other

- **Project Setup**
  - Added project logo and branding
  - Configured dual MIT/Apache-2.0 licensing
  - Updated to Dioxus 0.7.0-rc.0
  - Added comprehensive dependency management

- **UI/UX Improvements**
  - Migrated to Tailwind CSS v4 and DaisyUI v5
  - Added SVG icons and improved component styling
  - Enhanced responsive design and accessibility

- **Development Tools**
  - Added build optimization and release configuration
  - Configured cross-platform dependencies
  - Set up automated testing and benchmarking

### 🚜 Refactor

- Restructured project into workspace architecture
- Reorganized platform-specific modules for better maintainability
- Extracted reusable components and simplified component hierarchy
- Optimized asset management and dependency structure
- Improved code organization and removed unused dependencies

### 📚 Documentation

- Added comprehensive README with project overview and setup instructions
- Created demonstration GIFs and visual documentation
- Added development guides and contribution documentation
- Updated project metadata and license configuration

### ⚡ Performance

- Added benchmarking for bibliography reading operations
- Optimized chunk merging and parallel processing algorithms

### 🎨 Styling

- Enhanced component styling with modern design system
- Improved visual hierarchy and accessibility
- Streamlined CSS utilities and removed unused styles

### 🧪 Testing

- Added comprehensive test suite for core functionality
- Implemented UI testing for math rendering components

### ⚙️ Miscellaneous Tasks

- Set up automated release workflow
- Configured development environment and tooling
- Optimized build configuration for production
- Added editor configurations for better development experience

<!-- generated by git-cliff -->
