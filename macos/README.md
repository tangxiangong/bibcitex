# macOS native app

`BibCiTeX` is now the source target of the standalone **BibCiTeX** SwiftUI executable, including the main workbench and Spotlight panel. It links the shared Rust `bibcitex-ffi` static library through generated UniFFI 0.32.2 Swift bindings (`BibCiTeXCore`); there is no web host. Minimum macOS is 13. Glass uses `NSGlassEffectView` on macOS 26+, with native `NSVisualEffectView` on earlier systems. Existing bibliography settings are read by Rust.

The main window supports library registration/removal, native `.bib` picker, field/type search, metadata and BibTeX inspection, citation copying, and opening attachments/links. Closing it leaves the menu-bar app running. Cmd+Shift+K opens the helper; Arrow keys, Return, Tab and Escape preserve IME composition. xpaste remains in Rust and captures the foreground target before activating the panel.

Run `./macos/build.sh` yourself to create `dist/macos/<architecture>/BibCiTeX.app`. The script never launches it. The script uses SwiftPM’s Xcode build system and validates generated resource accessors before putting the application/MathJax resource bundles in `Contents/Resources`. Set `ARCH=arm64` or `x86_64`, `CONFIGURATION=debug` or `release`. Xcode 26+ and the corresponding Rust target are required. Dependencies and Swift bindings are resolved/generated only when you run the build. Set `BUILD_ONLY=1` to stop after compilation and resource validation, before bundle creation or signing. Release CI may supply a numeric three-component `APP_VERSION` and numeric `BUILD_NUMBER`. Generated sources are not handwritten or checked in; the Rust records and exported functions are the binding source of truth. SwiftPM resolves the pinned direct dependency versions when you run the build; its local Package.resolved is not tracked.

## Native updates

[Sparkle 2.10.0](https://github.com/sparkle-project/Sparkle/releases/tag/2.10.0) handles update discovery, the native update dialog, EdDSA verification, installation and relaunch. Release notes are disabled (`SUShowReleaseNotes=false`) so no HTML view is part of the application update UI. No feed URL or signing key exists in this repository. **检查更新 remains disabled until a real HTTPS appcast URL and Ed25519 public key are configured.** Set `SPARKLE_FEED_URL`, `SPARKLE_PUBLIC_ED_KEY` and `CODE_SIGN_IDENTITY` when manually building a distribution; ad-hoc signing is only for local use. Distribution also requires Developer ID signing, notarization, signed appcast archives and an HTTPS feed operated by the publisher. Do not use former web-host update artifacts.

See [Sparkle's publishing instructions](https://sparkle-project.org/documentation/publishing/) for signing update archives and generating appcasts. The build script bundles Sparkle's XPC services and signs nested services before the application. Updating has not been tested against a live signed feed.

## Verification

The arm64 SwiftUI executable has been compiled and linked with the actual UniFFI 0.32.2 output and Rust static library, targeting macOS 13. Helper model regressions and the generated Swift/Rust ABI integration tests have run successfully. Resource validation found the SVG and MathJax bundles, the Sparkle framework and the expected `Bundle.main.resourceURL` accessors. The GUI was not launched, and release packaging/signing and live update installation remain unverified.

Run `python3 macos/BibCiTeX/Tests/run-regressions.py` for the model tests. After a debug app/Rust build, run `python3 macos/BibCiTeX/Tests/run-interop-tests.py` for real generated-binding tests. The latter accepts `--products` and `--rust-library` for other build locations. These tests create only temporary bibliographies and do not modify the user's library registry or clipboard.


## Component limitation

For a field mixing mathematical chunks with paired literal dollar signs in ordinary text, LaTeXSwiftUI 1.5's string parser can interpret those literal signs as another formula. Version 2.0 fixes unmatched delimiters but does not expose typed text/math chunks. Plain fields use `Text(verbatim:)`; mixed fields disable Markdown formatting. No character-rewriting workaround, vendored renderer, or replacement math engine is used.
