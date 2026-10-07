# macOS

Open **`macos/BibCiTeX.xcodeproj`**, select the shared **BibCiTeX** scheme, and build. No `just` command, generated Swift file, C header, or prebuilt Rust library is required beforehand. Xcode 26+ and Rust installed through rustup are required; the app requires macOS 13 (Ventura) or later. The Xcode script discovers Cargo in the GUI's PATH or `$HOME/.cargo/bin` and installs a missing Apple target through rustup when needed.

The checked-in project has three real targets:

1. `RustBindings` builds `bibcitex-ffi` and runs the official UniFFI 0.32.2 generator.
2. `BibCiTeXCore` compiles the generated Swift interface into a static Swift library.
3. `BibCiTeX` compiles the SwiftUI/AppKit app and links both static libraries.

Debug and Release explicitly target macOS 13.0 for the project, app (including the helper), generated Swift core and Rust bindings. `LSMinimumSystemVersion` inherits that deployment target; it does not follow Xcode's recommended minimum. The Cargo default in `.cargo/config.toml` and all standalone Swift test executables also target macOS 13.0.

Target dependencies, declared script inputs/outputs and a discovered Rust dependency file control ordering and incremental rebuilds. All generated files and Rust compilation outputs live in DerivedData. The project resolves SwiftDraw, LaTeXSwiftUI and Sparkle through Xcode's Swift Package support; its shared `Package.resolved` pins transitive dependencies. SVG icons are app resources, MathJax resources are copied by Xcode's package integration, and Sparkle is embedded as a package framework. There is no second SwiftPM application build definition.

The main window includes library registration/removal, the `.bib` picker, field/type search, metadata and BibTeX inspection, citation copying, and attachment/link opening. Closing it leaves the menu-bar app running. Cmd+Shift+K opens the compact helper; Arrow keys, Return, Tab and Escape preserve IME composition. Rust xpaste captures the foreground target before activating the panel. Native glass uses `NSGlassEffectView` on macOS 26+, and `NSVisualEffectView` on earlier systems.

## Command line

The root Justfile invokes the same Xcode project:

- `just build [arch] [profile]` builds without exporting or signing a release.
- `just bundle [arch] [profile]` exports and signs `dist/macos/<architecture>/BibCiTeX.app`, without launching it.
- `just test-macos` runs update policy and generated Swift/Rust integration tests.

For a locally signed build, use `DEVELOPMENT_TEAM=YOUR_TEAM_ID just build arm64 debug` with an installed Apple Development certificate and its private key. Alternatively, pass `CODE_SIGN_IDENTITY` to use a specific installed identity. The wrapper enables signing when either is supplied; a requested signing failure is not retried unsigned. Without either, command-line builds remain unsigned for CI. In the Xcode app target, select your Team and Apple Development identity instead of the default ad-hoc identity (`-`).

On systems where `linkd` requires a validated bundle, unsigned/ad-hoc apps can log `com.apple.linkd.autoShortcut` error 4097 with `Unable to get teamId`. Setting a team string alone does not supply a certificate or establish a valid signature. This is separate from SwiftUI state-publication diagnostics.

Architectures are `arm64`, `x86_64`, or `universal`; profiles are `debug` and `release`. The `RustBindings` target builds the requested architectures and combines Rust static libraries with `lipo` for a universal build. Direct Xcode Release builds also support both standard architectures. The Bash wrapper accepts `DERIVED_DATA_PATH`; otherwise it uses `target/xcode/<architecture>`. CI can supply a three-component `APP_VERSION` and numeric `BUILD_NUMBER`. These scripts do not require Python, Ruby, XcodeGen, or another project generator.

Equivalent build command:

```sh
xcodebuild -project macos/BibCiTeX.xcodeproj -scheme BibCiTeX \
  -configuration Debug -derivedDataPath target/xcode/arm64 \
  -destination 'generic/platform=macOS' ARCHS=arm64 ONLY_ACTIVE_ARCH=NO \
  CODE_SIGNING_ALLOWED=NO build
```

## Updates

Sparkle 2 owns archive verification, automatic download/install and relaunch. The app menu exposes stable, beta and alpha channels and automatic download/install. Announcement feeds follow the application language (`zh-Hans` or `en`) and the running binary architecture. Native Markdown release notes are enabled; no WebView is added.

All update files are hosted in GitHub Releases. Versioned releases contain the DMG, `.app.zip`, localized notes and SHA256SUMS; the separate `update-feed` release contains architecture/channel/language appcasts. Formal releases can advance stable/beta/alpha feeds, beta can advance beta/alpha, and alpha only alpha. A feed never moves to an older semantic version.

The workflow does not require Apple Developer ID or notarization. It uses ad-hoc application signing plus a separate, self-generated Sparkle Ed25519 update key. Configure release-environment secret `SPARKLE_PRIVATE_ED_KEY` and variable `SPARKLE_PUBLIC_ED_KEY` from Sparkle's `generate_keys`; keep that key pair across releases. These are not paid certificates. Without a configured public key, local builds leave update checks disabled. macOS Gatekeeper may still require user approval for a non-notarized first installation; update verification does not remove OS checks.

See [the release protocol](../docs/updates.md). There is no legacy Tauri migration feed.

## Verification

The Xcode project has built the arm64 app with an empty DerivedData directory and no repository-local generated bindings. Default Debug architecture selection and the x86_64 command-line cross-build both pass. Cargo discovery also passes with a GUI-style minimal PATH. Generated Swift/Rust ABI tests pass against Xcode's static library products. An unchanged build skips the Rust binding script. Both architecture products contain all 31 SVG icons, MathJax resources and the embedded Sparkle framework; neither GUI was launched. Release distribution signing and notarization remain unverified.

UI/GUI tests are prohibited; GUI acceptance is manual. The standalone integration script is `BibCiTeX/Tests/run-interop-tests.sh`. It accepts `--products` and `--rust-library`; its default products are `target/xcode/<host-architecture>/Build/Products/Debug`, and it discovers generated headers and the Rust archive beside those products. Tests use temporary directories and retain a 30-second timeout; they do not change the user's registry or clipboard.

## Component limitation

A field mixing mathematical chunks with paired literal dollar signs in ordinary text can be misinterpreted by LaTeXSwiftUI 1.5's string parser. Version 2.0 fixes unmatched delimiters but does not expose typed text/math chunks. Plain fields use `Text(verbatim:)`; mixed fields disable Markdown formatting. No character-rewriting workaround, vendored renderer, or replacement math engine is used.
