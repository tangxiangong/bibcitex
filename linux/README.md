# Linux client

**Work in progress / 半成品：仍有许多未解决的问题，尚未完成 Linux 桌面验收。**
The current implementation is an unfinished development snapshot, not a complete
or production-ready Linux client. UI layout, tray behavior, helper/paste and
Wayland integration still need fixes and native desktop validation. The GNOME
extension and recent logo rendering changes have not been accepted on Ubuntu.
Builds and non-GUI tests do not establish runtime correctness or parity with
macOS and Windows.

BibCiTeX's Linux client uses **GPUI Kit 0.7.1** and calls `bibcitex-service`
directly. The macOS client is the reference for its workbench, compact helper,
independent tray workbench, terminology, and interactions. SwiftUI and WinUI
remain the platform implementations on macOS and Windows.

## Native layout reference

The interface is composed from GPUI Kit components: `TitleBar`, `Button`, `Icon`,
`Input`, `Textarea`, `Form`, `Select`, `Switch`, `Spinner`, menus, and resizable
panels. Icons load the existing standalone SVG files in `public/icons/`.

- The main workbench keeps the native 220-point sidebar and 350-point inspector,
  including inline library renaming, reference context menus, and a single row of
  inspector actions. List and detail copy feedback are independent.
- The helper uses the existing native metrics: a 720-point maximum width,
  56-point search header, 80-point reference rows, 56-point library rows, and a
  460-point maximum list. Its borderless surface grows with the number of results
  and returns to the search header when cleared.
- The independent tray workbench uses a 680-by-580-point surface, an integrated
  search/library selector, and a 282-point detail overlay with its own close and
  copy controls.
- Settings use a 480-by-480-point window with a Kit title bar and grouped form. Library editing uses a
  520-point-wide form with a file-selection card and multiline description.

Source alignment and compilation are separate from platform GUI acceptance.

## Build and install

Use stable Rust. Ubuntu 24.04 build dependencies:

```sh
sudo apt-get install clang libclang-dev cmake ninja-build pkg-config libssl-dev \
  libfontconfig1-dev libfreetype6-dev libxkbcommon-dev libxkbcommon-x11-dev \
  libwayland-dev libx11-xcb-dev libxcb1-dev libvulkan-dev libasound2-dev
cargo build --locked --release -p bibcitex-linux
cargo run --locked -p xtask -- package-linux target/release/bibcitex target/linux-packages arm64
```

Use `x86_64` instead of `arm64` on an x86-64 host. `just build debug` builds the
Linux development executable; `just bundle` builds release packages. Cargo and
IDEs do not require a prior Just invocation or generated language bindings.

Packaging produces `.deb`, `.tar.gz`, and Linux update metadata. The Debian
package targets Ubuntu 24.04-compatible library versions. Install it with
`sudo apt install ./BibCiTeX-VERSION-linux-ARCH.deb`. Extract the portable archive
and run its `install.sh` for a user-local installation, or run `bin/bibcitex`
directly. The installer adds the application icon, desktop launcher, and helper
and tray actions. It does not start the application.

## Current implementation (incomplete and not fully validated)

- Three-pane workbench with adjustable widths, persistent sidebar/inspector
  visibility, virtualized references, background search, field/type filtering,
  stale-result protection, and keyboard selection.
- Library add/edit/rename, pin/unpin, open file, and removal through the shared
  registry. Removing a library does not delete its bibliography file.
- Citation/BibTeX copying, selectable metadata, DOI/URL/file actions, typeset
  mathematical chunks, shared Chinese/English catalogs, and light/dark/system
  appearance. Math uses embedded STIX Two Math and local Rust SVG rendering;
  unsupported TeX retains its source instead of losing content.
- A compact search helper: its idle state is a search bar; typing expands results;
  Tab selects a library; up/down select; Enter or a row click pastes; Escape closes.
  Copy buttons remain available when desktop input permission or focus restoration
  fails. IME composition retains control of its keys.
- StatusNotifier tray icon/menu and a separate compact tray workbench with library
  switching, search, filters, details, refresh, and a main-window action.
- Single-instance activation using a private per-user Unix socket and a held file
  lock. Second launches forward commands, including Wayland activation tokens.
  Closing a window leaves the shortcut/tray service running. Quit from the tray,
  Ctrl+Q, or `bibcitex --quit`.
- Signed updates with stable/beta/alpha channels, periodic checks, an available
  update indicator, and portable automatic installation. COS is primary; GitHub
  is the fallback. Both archive and Debian payload hashes are authenticated.

Commands: `bibcitex`, `--helper`, `--tray`, `--settings`, `--quit`, `--version`,
`--help`. Ctrl+F focuses search; the default helper shortcut is Ctrl+Shift+K.

Preferences live in `$XDG_CONFIG_HOME/bibcitex/linux.json`, separately from the
shared bibliography registry. Invalid preference files are reported, not replaced.

## Desktop integration requirements

X11 registers the global shortcut through XGrabKey. It captures the previous
external window and process, requests focus, checks the target again, waits for
modifier release, then injects Ctrl+V through XTest. A shortcut conflict is reported
without disabling library management or clipboard copying.

Wayland registers shortcuts through the GlobalShortcuts portal and consumes
compositor-issued activation tokens. Automatic paste requests keyboard access
through RemoteDesktop, closes the helper so the compositor can restore focus,
then sends the paste chord. Access is subject to the user's authorization and the
desktop's portal implementation. The app does not bypass those policies or claim
success when an interface rejects the request. It refuses to inject into an owned
application window when no external target was captured. Unlike X11, portable
Wayland APIs do not expose the identity of another app's focused window, so actual
focus restoration must be accepted on each supported compositor.

Tray activation preserves the host's screen-coordinate hint and selects that
monitor. X11 uses those coordinates; Wayland compositors advertising Layer Shell
use an anchored layer surface. GNOME Wayland does not provide this positioning
protocol, so ordinary client-side window bounds cannot place the tray workbench
beside the icon there. Ubuntu's AppIndicator extension also handles single left
and right clicks as menu actions, reserving activation for a double click; the
application's `ItemIsMenu=false` cannot override that host behavior. Matching the
native clients on GNOME requires Shell-side integration, not just an SNI callback.

The bundled `bibcitex-tray@tangxiangong.github.io` GNOME Shell extension supplies
that integration for GNOME 45–50, alongside Ubuntu AppIndicators (or AppIndicator
and KStatusNotifierItem Support). It intercepts only BibCiTeX's primary click;
right-click menus and other applications remain with the existing host. It moves
only the `BibCiTeX Tray` window with BibCiTeX's application ID. Disabling it removes
its event handlers. Its supported-version metadata is not desktop acceptance.

The `.deb` and portable installer copy the extension but never enable it. After
installing, log out and back in so GNOME discovers the new extension, then run:

```sh
gnome-extensions enable bibcitex-tray@tangxiangong.github.io
```

For a source checkout, copy the extension first (no application packaging needed):

```sh
extension_dir="${XDG_DATA_HOME:-$HOME/.local/share}/gnome-shell/extensions/bibcitex-tray@tangxiangong.github.io"
mkdir -p "$extension_dir"
cp linux/gnome-extension/extension.js linux/gnome-extension/metadata.json "$extension_dir/"
```

Then log out/in and enable it as above. To restore the host's default behavior:
`gnome-extensions disable bibcitex-tray@tangxiangong.github.io`.

The tray needs a StatusNotifier host; desktops without one keep the main-window
and shortcut entry points available. GPU, input-method, accessibility, focus,
clipboard, scaling, and tray behavior require desktop acceptance in addition to
compilation. No GUI or interface-model automation tests are part of this project.

## Updates and release configuration

Set `BIBCITEX_LINUX_UPDATE_PUBLIC_KEY` to the base64 32-byte Ed25519 public key
when building a release. The release environment uses the same-named GitHub
variable and a `LINUX_UPDATE_PRIVATE_KEY` secret containing the base64 32-byte
private seed. No private key is compiled into the client. Builds without a public
key can run, but cannot accept updates.

The release workflow builds Linux x86-64 and ARM64 packages, signs each architecture's
metadata, verifies both architectures and both payload formats before publication,
and mirrors the artifacts to COS alongside the existing platform releases. Older
release repair workflows remain compatible with tags that have no Linux client.

Portable updates verify Ed25519 metadata, exact version/architecture/file names,
size and SHA-256, then replace only `bin/bibcitex` atomically. Settings offer restart
after installation. System installs under `/usr`, `/opt`, or `/app` instead download
the verified Debian package and hand it to the system package installer; automatic
in-place replacement is disabled for those locations.

## Validation

`just check-rust`, `just test-ci`, Linux native Clippy/tests/build, and actual
package-content/portable-install checks cover compilation, service semantics,
single-instance IPC, signing, version policy, and installation mechanics. These
checks do not launch the GPUI application and do not prove GNOME/KDE desktop
interaction, real cross-application paste, or signed production update delivery.
The check/release workflows include native Ubuntu x86-64 and ARM64 jobs.
