# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0-alpha.0] - 2025-08-14

### 🚀 Features

- Add logo and update theme to light mode with simplified styling
- Adds inline math component
- Adds custom error type for bibcitex-core
- Adds bibtex parsing functionality
- Add bib and error modules
- Add `setting` module for managing bibliographies
- Implement database management components
- Add home view with database component
- Add navigation bar component
- Add home and nav modules
- Add routing
- Add global state management
- Track bibliography modification times
- Add detail page and link bibliography items to it
- Add empty reference components for bibliography entries
- Add rayon for parallel processing
- Introduce reference and article structs
- Add utils module with parallel bibliography reading functionality
- Add automatic parallel reading for large bibliographies based on size threshold
- Add global signal for current reference
- Implement reference entry display with styling and math support
- Add copy to clipboard functionality for cite key
- Implement reference search functionality with parallel processing support
- Add hover tooltip to display reference details on mouseover
- Initialize the framework of the spotlight-like citation  helper tool
- Add enigo keyboard automation dependency to project
- Add bibliography selection interface to helper window
- Add keyboard navigation and smooth scrolling for search results
- *(ui)* Add tailwindcss support and improve bibliography styling
- *(build)* Add parallel serve command combining css-watch and dx-serve
- Add field-specific reference search with filter controls
- Add reference type filtering and improve search UI
- Add path abbreviation utility function
- Add URL, file and source fields to Reference struct
- Add MSC code parsing and lookup functionality
- Add abstract field to Reference struct
- Add edition, issue and book pages fields to Reference struct
- Add support for school and address fields in Reference
- Replace JavaScript scroll with native Dioxus scroll API
- Update global hotkey and window handling
- Add book title, editor, month and organization fields
- Add support for BibTeX institution field
- Initialize module for platform-specific macOS window functionality
- Add optional description field to bibliography info
- Add support for eprint and arxiv fields in Reference struct
- *(ui)* Add support for Misc and arXiv reference types
- *(utils)* Add `observe_app` and `focus_previous_window` functions
- *(windows)* Add windows platform support for paste functionality
- Update app title and window configuration
- *(shortcuts)* Add shift modifier to helper hotkey
- *(linux)* Add linux platform support for window focus and paste
- *(bibliography)* Add error handling with fade animation and visual indicators
- *(components)* Improve focus handling and styling in Select and Search
- *(windows)* Add windows subsystem feature for bundling
- *(theme)* Replace nord theme with winter and dracula themes
- *(helper)* Add window drag support and disable resizing
- *(search)* Support hyphen-separated title queries

### 🐛 Bug Fixes

- Normalizes cite key search to lowercase
- Fix pages field name in biblatex entry parsing
- Extend thesis filter to include all thesis types
- Fix CSS color class grey to gray
- Move katex to target-specific dependencies
- Preserve clipboard content after paste operation
- Handle potential None case in CURRENT_REF calls
- *(windows)* Improve window focus handling and paste reliability
- *(references)* Re-run search when filter type changes
- *(helper)* Adjust window height calculation by removing extra padding

### 💼 Other

- Adds new logo and updates readme
- Adjust logo text positioning for better letter spacing
- Update application icons
- Adds `katex` crate for rendering LaTeX formula
- Add `biblatex`, `fs-err`, `thiserror` dependencies
- Add bibcitex-core dependency and rfd
- Add `chrono` and `itertools` dependencies
- Add biblatex as a dependency
- Add clipboard support and tokio runtime
- Updates TODO list
- Update @tailwindcss/cli dependency to v4.1.11
- Style entry display and hover tooltip components
- Remove hardcoded colors for DaisyUI theme support
- Update CSS with Nord theme and HTML customization
- Improve bibliography header styling
- Remove custom CSS in favor of DaisyUI classes
- Add select component and improve reference view styles
- Add SVG icons for basic actions
- Add tooltip and improve bibliography UI
- Add spotlight shortcut button to navigation bar
- Add delete icon and update bibliography layout styling
- Remove unused bib.css stylesheet
- Refactor bibliography view to use table layout
- Add tooltip to home logo in navbar
- Add tooltips and file opening functionality to bibliography
- Add link styles and right-aligned tooltips
- Add tooltip to delete button and improve button styles
- Remove unused CSS files
- Add copy ico
- Move static assets to lib.rs
- Add Article card (unimplemented)
- Update css
- Add DOI badge with browser open link functionality
- Update bibliography component button styles
- Add margin and padding utility classes
- Update article component styling and links
- Remove card component and add new design tokens
- Replace hover with drawer component for reference details
- Replace hover with drawer component for reference details
- Replace hover with drawer component for reference details
- Update css
- Update `daisyui` to `v5.0.50`
- Add CSV, once_cell and update criterion dependency
- Add MR Class code support and improve article details
- Improve reference drawer title and styling
- Update detail.svg icon and add collapse styles
- Add support for books in reference component
- Add support for book entry type in references view
- Remove unused styles and color tokens
- Refactor Article and Book component helpers
- Add gray and text utilities to Tailwind config
- Fix filter dropdown layout and search input width
- Fix horizontal scrolling issues
- Add dual MIT/Apache-2.0 licensing
- Add thesis reference type support
- Add pink and rose color variables and utilities
- Redesign helper window UI with Tailwind CSS
- Remove helper.css and enhance Tailwind styles
- Add Spotlight-style transparent window and UI polish
- Add Spotlight-style blur and scroll blocking utilities
- Fix layout overflow with proper flex container setup
- Fix horizontal overflow issues sitewide
- Remove backdrop blur effect from UI elements
- Remove unused backdrop-filter and blur styles
- Add logo and simplify search component structure
- Add CSS transform properties and border radius styles
- Simplify reference entry layout and styling
- Remove "et al." truncation and background colors for authors
- Add new color and display utility classes
- Add dynamic height adjustment for helper windows
- Fix search box keyboard navigation and scrolling
- Add InProceedings UI components and citation handling
- Add purple color variations and utility classes
- Add support for TechReport entry type
- Add amber color CSS variables and related utility classes
- Add menu, mask and status component styles
- Update dependencies and fix KaTeX config
- Add w-1/2 Tailwind utility class
- Add missing color variables and utility classes
- Add badge styling to reference component metadata
- Add badge component styles and fix join spacing
- Update tailwind
- Add windows-sys dependency for Windows target
- Add x11 dependency for linux target
- Update `dioxus` dependencies to 0.7.0-rc.0
- Optimize release build and update icon paths

### 🚜 Refactor

- Rename package name
- Restructure project into workspace and update logo asset path
- Use static for asset declarations
- Extract modal and navbar styles to separate CSS files
- Remove unused tailwind css asset and link
- Rename database component to bibliography with improved UI and state management
- *(views)* Rename detail to references and update related components
- Remove wildcard re-export of utils module
- Rename `icon` into `icon-gen`
- Adapt reference struct for dioxus props
- Remove unnecessary print statements
- Extract search functionality into separate SearchBib component
- Minify css
- Move index.html content inline to main.rs
- Fmt
- Rename input.tailwind.css to tailwind.css
- Refactor, Rename and Remove
- *(ui)* Refactor reference rendering to use ReferenceComponent
- Rename `tailwind.css` to `input.tailwind.css`
- Remove leading slashes from asset paths
- Refactor nested if-let patterns using && syntax
- *(platforms)* Reorganize macOS platform modules and add paste functionality
- *(macos)* Move MAIN_WINDOW_TITLE to shared module
- Remove unused profile sections and clean up web config
- *(reference)* Remove MRClass component and related functionality
- *(helper)* Simplify clipboard handling in search function
- *(math)* Replace katex with katex-gdef-v8 for rendering
- *(helper)* Extract window size constants and clean up CSS
- *(helper)* Adjust height calculations and margins for consistency
- Remove unused clipboard code and theme attributes
- Remove csv dependency and update objc2 version
- *(helper)* Split helper component into several sub-components
- *(icons)* Modify path for application icon assets in multiple sizes and formats

### 📚 Documentation

- Add project overview and setup instructions to README
- Add project overview and usage instructions to README
- Update project description to be more concise
- Add TODO list and update project documentation
- Add claude.ai dev guide
- Uncomment Tailwind docs and update for Bun usage
- Update `CLAUDE.md`
- Update README with full project documentation
- Update README and add assets
- Add TechReport and InProceedings support and licenses
- Remove text title
- Replace BibCiTeX with transparent logo
- Add Misc type and increase logo size to 50px
- Update copyright notices and attribution for EcoPaste code
- *(platforms)* Add linux paste support and update documentation
- *(prompts)* Add documentation files for claude and dioxus guidance
- *(readme)* Add demonstration gifs and logo for documentation
- Update README with platform support and UI previews
- Update readme gif assets
- Update readme gif assets

### ⚡ Performance

- Add benchmark for bibliography reading
- Optimize chunk merging with new merge_chunks utility function

### 🎨 Styling

- Update save button border
- *(components)* Improve reference card styling with badges and card layout
- *(css)* Add new animations and UI components
- *(tailwind)* Add w-1/6 utility class and remove w-fit
- *(bibliography)* Remove fixed column widths and divider in table
- *(helper)* Improve reference item styling and author display
- *(tailwind)* Remove unused divider and success classes, add hover utilities
- *(helper)* Fix formatting of long string literals in Search component
- *(tailwind)* Update background color classes and hover states
- *(reference)* Update styling for no title text in reference components
- *(components)* Update text colors for better visual hierarchy
- *(tailwind)* Update and clean up CSS variables and utility classes

### 🧪 Testing

- *(ui)* Adds test components for math rendering
- Removes test files
- Improve setting tests
- Remove builder pattern implementation from macOS panel

### ⚙️ Miscellaneous Tasks

- Initialize project with basic Dioxus setup and app icons
- Add dependencies
- Update app identifier and title to BibCiTeX in bundle config
- Update cargo-sort to sort workspace packages
- Add .bib extension to .gitignore
- Update dependencies
- Remove Tailwind CSS configuration and stylesheet files
- Remove unused anyhow dependency from workspace
- Adds criterion benchmark
- Update dioxus and redox_syscall dependencies
- Optimize release profile for size and performance
- Comments out tailwind installation guide
- Switch from dual MIT/Apache-2.0 to MIT-only license
- Add Tailwind CSS setup and configuration
- Add Justfile for running Tailwind CSS
- Upgrade to Tailwind CSS v4
- Cleanup formatting tools and configs
- Update gitignore and add tailwind css configuration without node env
- Add css-minify command to Justfile
- Switch from scripts/tailwindcss-extra to Bun
- Remove postcss-purgecss dependency
- Minify
- Format Tailwind CSS
- Rename `assets/icons` directory to `assets/app-icons`
- Init Objective-C bindings for NSPanel/NSWindow behavior
- Extract macOS panel code into separate nspanel crate
- Exclude nspanel dir from GitHub language stats
- Remove
- Add opener crate for opening files in default applications
- Add Zed editor settings for Rust and Tailwind
- Add commands to generate desktop app icons
- Add TailwindCSS editor configuration for vscode
- Update Dioxus to 0.7
- Add macOS dependencies and pastey crate
- Add NOTICE file documenting third-party code usage
- Update Tailwind class regex patterns for better matching
- Add once_cell dependency
- Remove clippy.toml configuration file
- Update dependencies including syn, thiserror, and others
- Add GitHub workflow for automated releases

<!-- generated by git-cliff -->
