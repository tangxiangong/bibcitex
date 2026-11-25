# BibCiTeX NSH desktop client

`linux/` is the **NSH 0.7.0 mixed Rust + TSX client for macOS, Windows and
Linux**. `App.tsx` owns the windows, stores and existing native-style UI; Rust
exports library, search, tags, preferences, math and paste operations through
`bibcitex/native`. The shared `bibcitex-service` remains the business core.
The separate SwiftUI macOS and WinUI Windows clients and their recipes remain
available; these NSH packages are built independently.

Compilation and packaging do not establish GUI parity, real cross-application
paste, compositor acceptance, or successful production update delivery.

## Commands

Install the NSH CLI/runtime and stable Rust. CI downloads the latest official
NSH release. From the repository root:

```sh
just nsh-check
just nsh-build
just nsh-package macos
just nsh-package windows
just nsh-package linux
```

Run each package build on its platform with that platform's native toolchain and
NSH components. The local validation below covers macOS packaging, not Windows or
Linux packages. `just nsh-run` launches a development GUI only when explicitly
requested. Packaging never passes `--run`.

The equivalent commands, from `linux/`, are:

```sh
nsh check
nsh build
nsh package --target macos
# On the corresponding platform:
nsh package --target windows
nsh package --target linux
```

The output is under `linux/.nsh/out/`: macOS produces `BibCiTeX.app` and an
update `.tar.gz`, Windows a runner directory and `.zip`, and Linux a runner
bundle, `.desktop` file and `.tar.gz`. NSH's macOS package is ad hoc signed;
Developer ID signing/notarization and publication are separate release work.
It is not the previous hand-written Debian/portable updater pipeline.

`public/logo.png` is the existing 512×512 square brand artwork, equal in source
resolution to `public/favicon.png` and `assets/app-icons/icon.png`. The build
script stages it as `.nsh/resources/app-icon.png`; NSH creates the platform icon.
It is not upscaled or replaced by generated artwork.

## Generated assets and localization

The root `localization/en.json` and `localization/zh-Hans.json` are the **only
editable text catalogs**. The Cargo build script `src/assets_build.rs`:

- stages existing artwork/SVGs inside `.nsh/resources/`;
- generates ignored `locales/en.json` and `locales/zh-Hans.json`;
- generates `.nsh/generated/config.ts` from `nsh.json` for version/channel policy.

Do not edit or commit these outputs. The build script watches the sources and
outputs, so deleting a generated directory or editing a root catalog regenerates
it. NSH's mixed-mode Cargo check/describe step runs the build script before TS
checking, locale bundling and package icon loading. No prior Just command or
manual asset bootstrap is required. This order was checked on a source copy
starting with neither `locales/` nor `.nsh/`.

The compiler encodes every source key as `k` followed by four lowercase hex
characters per UTF-16 code unit. This reversible encoding preserves Chinese,
periods, colons, spaces, parentheses and supplementary Unicode without relying
on rust-i18n's dotted-key flattening. The frontend mirrors the encoding; screens
continue calling `tr(originalKey, ...args)`.

Numbered `{0}`, `{1}`, … markers become `%{p0}`, `%{p1}`, … . Original `%`
characters become `%{pct}` and are restored by interpolation. Argument values
are inserted once, without reinterpreting `%{...}` inside them; omitted numbered
arguments retain their original `{n}` form. Non-UI tests run every root catalog
entry through NSH's actual rust-i18n loader and runtime translator, verify
reversible keys, complete entry counts and interpolation, and exercise literal
percent patterns and astral Unicode explicitly.

`src/i18n/index.ts` delegates to `nsh/i18n` (`t`, `setLocale`,
`availableLocales`). There is no `L10n` bridge or startup dictionary transfer. A single
`DesktopSession.systemLocale` OS tag prevents NSH's language override from being
mistaken for the underlying system language when switching back to System.
For the system preference, `zh`, `zh-Hans`, `zh-CN` and `zh-SG` select `zh-Hans`;
Traditional Chinese and other languages select English. `setLocale` keeps
translations, NSH `locale()` and `Intl` in the same selected language. The root
catalogs and their existing UI wording remain unchanged.

## OTA code updates

The packaged NSH host reads the signed code feed at
`/bibcitex/nsh/ota/1/latest.json` on the release COS bucket. The manifest
public key and trusted package root ship in `nsh.json`. Install a host built
with these keys once to receive subsequent TSX and asset changes over the air.

The client checks for OTA code ten seconds after startup and every twelve hours
after a successful check. A newer compatible build downloads in the background,
even when the app version stays the same. NSH verifies the package and loads it
on the next natural launch. The current session continues uninterrupted. Failed
checks and downloads retry after fifteen minutes. The Settings switch, channel
selector and manual restart action apply to whole-app updates only.

When releasing the v0.7 host, use a build number higher than any shipped v0.6
OTA build. This ensures the host starts with the new window entry point even
when an older OTA package is already installed.

### Publish an OTA build

Dispatch **Publish NSH code update** on the revision to distribute. Supply a
build number higher than the host and previous OTA builds, and release notes.
The signing CLI sets its runtime version as the package minimum.
The workflow signs a `.nshpkg` and its manifest, uploads the immutable package
to COS, then advances the feed. A new application version or GitHub Release is
not required. The feed directory `1` identifies the native bridge compatibility
generation. If new code requires exports missing from installed hosts, ship a
host with a new feed directory before publishing code for that generation.

The GitHub `release` environment holds `NSH_OTA_MANIFEST_KEY` and
`NSH_OTA_RELEASE_KEY` as secrets and `NSH_OTA_RELEASE_CERT` as a variable.
The root private key is kept outside CI. Renew the app-scoped release
certificate with the root before expiration. Hosts trust that root and can
accept a renewed release certificate without a new installation.

## Desktop integration

The main workbench, compact helper, independent tray panel, settings and about
window share one NSH engine. The helper captures its external target before
opening, prepares the exact formatted citation, hides, commits once, and keeps
failed citations available for manual copy. Rust owns Linux X11/portal shortcuts
and StatusNotifier integration; NSH owns macOS/Windows tray and global shortcuts.
`statusBar`, `application`, window, clipboard and HTTP(S) URL capabilities are
explicitly declared.

Linux uses GTK's clipboard provider and `xpaste::linux::Paste` for target tracking,
XTest and Wayland RemoteDesktop permission/session management. X11 and Wayland
acceptance still requires a real desktop. The optional GNOME extension in
`gnome-extension/` uses the fixed `BibCiTeX Tray` title and application ID; it is
not automatically enabled or installed by the NSH packager. Install/enable it
manually only when requested. Other compositors apply their own placement and
activation policies.

## Scoped validation

```sh
cargo test --locked -p bibcitex-linux --lib
cargo clippy --locked -p bibcitex-linux --lib --tests -- -D warnings
cargo fmt -p bibcitex-linux -- --check
cargo check --locked -p bibcitex-linux --lib --target x86_64-pc-windows-gnu
just --fmt --check
git diff --check
```

No UI/interface-model tests are added. The macOS `.app` check does not launch the
app, and does not prove Windows/Linux GUI behavior, notarization or a published
update's end-to-end delivery.
