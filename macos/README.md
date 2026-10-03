# macOS

Open **`macos/BibCiTeX.xcodeproj`**, select the shared **BibCiTeX** scheme, and build. No `just` command, generated Swift file, C header, or prebuilt Rust library is required beforehand. Xcode 26+ and Rust installed through rustup are required; the deployment target remains macOS 13. The Xcode script discovers Cargo in the GUI's PATH or `$HOME/.cargo/bin` and installs a missing Apple target through rustup when needed.

The checked-in project has three real targets:

1. `RustBindings` builds `bibcitex-ffi` and runs the official UniFFI 0.32.2 generator.
2. `BibCiTeXCore` compiles the generated Swift interface into a static Swift library.
3. `BibCiTeX` compiles the SwiftUI/AppKit app and links both static libraries.

Target dependencies, declared script inputs/outputs and a discovered Rust dependency file control ordering and incremental rebuilds. All generated files and Rust compilation outputs live in DerivedData. The project resolves SwiftDraw, LaTeXSwiftUI and Sparkle through Xcode's Swift Package support; its shared `Package.resolved` pins transitive dependencies. SVG icons are app resources, MathJax resources are copied by Xcode's package integration, and Sparkle is embedded as a package framework. There is no second SwiftPM application build definition.

The main window includes library registration/removal, the `.bib` picker, field/type search, metadata and BibTeX inspection, citation copying, and attachment/link opening. Closing it leaves the menu-bar app running. Cmd+Shift+K opens the compact helper; Arrow keys, Return, Tab and Escape preserve IME composition. Rust xpaste captures the foreground target before activating the panel. Native glass uses `NSGlassEffectView` on macOS 26+, and `NSVisualEffectView` on earlier systems.

## Command line

The root Justfile invokes the same Xcode project:

- `just build [arch] [profile]` builds without exporting or signing a release.
- `just bundle [arch] [profile]` exports and signs `dist/macos/<architecture>/BibCiTeX.app`, without launching it.
- `just test-macos` runs helper and generated Swift/Rust integration tests.

Architectures are `arm64`, `x86_64`, or `universal`; profiles are `debug` and `release`. The `RustBindings` target builds the requested architectures and combines Rust static libraries with `lipo` for a universal build. Direct Xcode Release builds also support both standard architectures. The Bash wrapper accepts `DERIVED_DATA_PATH`; otherwise it uses `target/xcode/<architecture>`. CI can supply a three-component `APP_VERSION` and numeric `BUILD_NUMBER`. These scripts do not require Python, Ruby, XcodeGen, or another project generator.

Equivalent build command:

```sh
xcodebuild -project macos/BibCiTeX.xcodeproj -scheme BibCiTeX \
  -configuration Debug -derivedDataPath target/xcode/arm64 \
  -destination 'generic/platform=macOS' ARCHS=arm64 ONLY_ACTIVE_ARCH=NO \
  CODE_SIGNING_ALLOWED=NO build
```

## Updates

[Sparkle 2.10.0](https://github.com/sparkle-project/Sparkle/releases/tag/2.10.0) handles update discovery, its native dialog, EdDSA verification, installation and relaunch. `SUShowReleaseNotes=false` disables the HTML release-notes view. **检查更新 remains disabled until a real HTTPS appcast URL and Ed25519 public key are configured.** The bundle command accepts `SPARKLE_FEED_URL`, `SPARKLE_PUBLIC_ED_KEY` and `CODE_SIGN_IDENTITY`; ad-hoc signing is for local use. Distribution requires Developer ID signing, notarization, signed update archives and a publisher-operated HTTPS feed.

See [Sparkle's publishing instructions](https://sparkle-project.org/documentation/publishing/). The export script signs nested services before the app and preserves Downloader entitlements. Live update installation has not been tested.

## Verification

The Xcode project has built the arm64 app with an empty DerivedData directory and no repository-local generated bindings. Default Debug architecture selection and the x86_64 command-line cross-build both pass. Cargo discovery also passes with a GUI-style minimal PATH. Helper model regressions and generated Swift/Rust ABI tests pass against Xcode's static library products. An unchanged build skips the Rust binding script. Both architecture products contain all 31 SVG icons, MathJax resources and the embedded Sparkle framework; neither GUI was launched. Release distribution signing and notarization remain unverified.

The standalone test scripts are `BibCiTeX/Tests/run-regressions.sh` and `BibCiTeX/Tests/run-interop-tests.sh`. The latter accepts `--products` and `--rust-library`; its default products are `target/xcode/<host-architecture>/Build/Products/Debug`, and it discovers generated headers and the Rust archive beside those products. Tests use temporary directories and retain 20/30-second timeouts; they do not change the user's registry or clipboard.

## Component limitation

A field mixing mathematical chunks with paired literal dollar signs in ordinary text can be misinterpreted by LaTeXSwiftUI 1.5's string parser. Version 2.0 fixes unmatched delimiters but does not expose typed text/math chunks. Plain fields use `Text(verbatim:)`; mixed fields disable Markdown formatting. No character-rewriting workaround, vendored renderer, or replacement math engine is used.
