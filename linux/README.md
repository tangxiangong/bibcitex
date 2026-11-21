# BibCiTeX NSH desktop client

`linux/` is the **NSH 0.6.1 mixed Rust + TSX client for macOS, Windows and
Linux**. `App.tsx` owns the windows, stores and existing native-style UI; Rust
exports library, search, tags, preferences, math and paste operations through
`bibcitex/native`. The shared `bibcitex-service` remains the business core.
The separate SwiftUI macOS and WinUI Windows clients and their recipes remain
available; these NSH packages are built independently.

Compilation and packaging do not establish GUI parity, real cross-application
paste, compositor acceptance, or successful production update delivery.

## Commands

Install the matching NSH CLI/runtime and stable Rust. From the repository root:

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

## Updates

`src/store/updater.ts` owns the reactive update state for Settings, the main
window badge, menus and tray actions. It uses `nsh/updater` directly:

- `shellInfo().version` distinguishes an installed package from `nsh run`;
  `runningPackage()` identifies separately updated code when present.
- Manual checks pass the selected locale for release notes. Stable uses the
  default GitHub source unless a signed stable channel is declared. An undeclared
  beta/alpha channel shows the existing unavailable message instead of crashing.
- Downloads report received/total bytes. A writable installation is verified and
  staged with `installOnQuit()`; the existing restart action calls
  `installAndRestart()`. A non-writable installation opens the update's URL.
- Installed packages check after 10 seconds and every 12 hours. With
  `automaticUpdates` enabled, installable updates download and stage for quit.
  Development runs do not perform background update work.
- The channel selector is disabled during a check/download or while an update
  waits for installation. Checks and downloads also wait for optimistic
  preference writes to settle, so a failed channel save cannot stage a package
  from the reverted channel. Preferences still persist `updateChannel` and
  `automaticUpdates`. Errors and progress use existing root catalog text.

The current source is configured as:

```json
"updater": { "github": "tangxiangong/bibcitex" }
```

NSH reads `latest.json` from that repository's latest GitHub Release. This code
connects the client APIs; a compatible manifest and archives still have to be
published. Existing native Sparkle/Velopack feeds are not NSH manifests. The
GitHub source permits unsigned metadata until a real `publicKey` is configured;
no signing key has been invented, generated, or embedded by this change.

### Signed multi-channel publication

Before a production signed release, the release owner creates a real key in
secure storage (do not run this in the repository or commit the secret):

```sh
nsh keygen /secure/release/bibcitex-update
```

Keep the resulting `.secret` offline or in the release secret store. Put the
actual contents of its `.pub` in `updater.publicKey`. Only then declare
`updater.channels`, mapping `stable`, `beta` and `alpha` to the release owner's
real HTTPS manifest URLs. Do not put placeholder keys/URLs in the app's live
configuration. NSH rejects channels without `publicKey`. Rebuild packages to
embed the trusted key and channel URLs before those channels can be selected.

Build each target with `nsh package --target <platform>` and collect the archives
named `BibCiTeX-VERSION-OS-ARCH.ext`. From the repository root, use the real
archive paths and notes files for that release, for example:

```sh
nsh update-manifest --entry linux \
  --key /secure/release/bibcitex-update.secret \
  --notes release-notes/VERSION/default.md \
  --notes en=release-notes/VERSION/en.md \
  --notes zh-Hans=release-notes/VERSION/zh-Hans.md \
  --base-url https://github.com/tangxiangong/bibcitex/releases/download/RELEASE_TAG/ \
  --out target/nsh-release \
  PATH_TO_MACOS_ARCHIVE PATH_TO_WINDOWS_ARCHIVE PATH_TO_LINUX_ARCHIVE
```

`VERSION`, `RELEASE_TAG`, archive paths and note paths above are placeholders for
the release owner, not files this task generated. Publish the generated
`latest.json`, its signature, notes and all referenced archives together. For
beta/alpha, publish each manifest at its configured channel URL using the same
trusted key. Verify the manifest/platform identity, downloads and installation
on actual installed packages before claiming signed update deployment.

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
