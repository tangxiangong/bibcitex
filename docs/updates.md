# Native updates

macOS uses Sparkle 2; Windows uses Velopack 1.2.161. Neither runtime depends on release-hub, Tauri, a private server, or Microsoft Store. No old updater, settings or installer migration is performed.

## Channels and versions

Release tags must be `vMAJOR.MINOR.PATCH`, `vMAJOR.MINOR.PATCH-alpha.N`, or `vMAJOR.MINOR.PATCH-beta.N`. The Rust release validator uses SemVer, not lexical or publication-date order. Stable advances stable/beta/alpha audiences, beta advances beta/alpha, and alpha advances alpha, only when the new version is greater than each audience's current version. A user switching back to stable waits for a newer stable release; automatic downgrade is disabled.

User versions preserve the prerelease suffix. Sparkle uses a monotonically increasing CI build number for CFBundleVersion. MSI maps the CI run number to `1.(run / 65536).(run % 65536)`; build numbers must remain unique and increasing and be below 16777216. If moving to another release workflow, carry the sequence forward. Do not reuse published tags or rebuild their payloads. Publishing an older stable tag never replaces GitHub Latest. Windows publication requires an empty output directory and uses a fresh publish directory to exclude obsolete files.

## GitHub artifacts

Each versioned Release includes:

- `BibCiTeX-<version>-macos-{arm64,x86_64}.dmg`
- `BibCiTeX-<version>-macos-{arm64,x86_64}.app.zip` (contains the APP bundle)
- `BibCiTeX-<version>-windows-{x64,arm64}.{exe,msi}`
- Velopack full NUPKG files and architecture/channel JSON feeds
- Sparkle architecture feeds, `notes-zh-Hans.md`, `notes-en.md`
- `release.json`, `SHA256SUMS`

A dedicated prerelease named `update-feed` contains mutable channel indices, `channels.json`, and immutable full NUPKG files. Sparkle appcasts point directly to versioned release archives and notes. Velopack's SimpleWebSource reads a known channel filename and packages from this dedicated release, so it does not depend on the GitHub API's latest-release or recent-ten-release filtering.

First publish the versioned GitHub Release. Its `published` event starts builds from that tag; manual dispatch can target an existing public Release. All four targets must build and validate before attachment upload. The Release can be temporarily empty or incomplete, but update feeds do not advance until every required asset has uploaded. Retry skips existing assets with matching size and SHA-256 digest and uploads only missing files; conflicting or digest-less assets are never replaced. The inventory uploads last. Re-run failed jobs to retain the original packages rather than rebuilding a populated version. Full NUPKG files upload to update-feed before any channel index points to them. The workflow serializes release runs. Existing channels for newer versions are preserved. GitHub has no transaction across multiple assets: an interrupted index upload may leave some audiences on their preceding valid version; no index references a missing payload. Do not delete old NUPKG files while a feed or a running client may reference them.

## Announcements and UI

Provide both `release-notes/<version>/zh-Hans.md` and `en.md`. Release body and native announcements derive from these files. Windows embeds both languages into package Markdown with locale markers; the native renderer excludes HTML and remote images. macOS selects a localized `.md` URL through its architecture/channel/language appcast. No update web page or WebView is embedded.

Manual checks present available updates and errors. Windows checks at startup and every six hours; Sparkle manages the macOS schedule. Automatic installation is opt-in. Windows downloads and applies the specifically approved version at the next launch, while Sparkle manages automatic installation when possible. Changing the channel or automatic-update preference cancels a background download and invalidates its pending installation. Installer hooks run before preference loading, and a malformed settings file is preserved with a `.corrupt-<id>` suffix. System authorization prompts cannot be suppressed. Finish saving application state before a manual install/restart.

## Integrity and keys

Windows uses Velopack size/hash validation. Publication also requires SHA-256 for each full package. macOS requires an Ed25519-signed archive; publication verifies that signature against the configured public key. A release inventory and SHA256SUMS cover all artifacts. Hashes alone do not authenticate the publisher if the repository is compromised.

Generate a Sparkle update key once using the `generate_keys` binary delivered with the project's pinned Sparkle package. Store the exported private key in GitHub environment `release` secret `SPARKLE_PRIVATE_ED_KEY`, and its public key in variable `SPARKLE_PUBLIC_ED_KEY`. Never commit the private key. Retain the key for future updates. The build deliberately fails when release keys are missing or mismatched. These keys do not require Apple membership or a commercial certificate. App builds use ad-hoc signing; Gatekeeper and Windows SmartScreen may still require user action.

## Verification

Run `just check-rust`, `just test-ci`, platform builds and native interop tests. Release tests cover corrupted archives/packages, mismatched keys, missing targets/notes, malformed tags, channel promotion and upload failures. Windows semantic compilation on macOS checks C# types but does not replace Windows XAML compilation or installation testing. Real acceptance requires installing two actual release versions on Intel/Apple Silicon Macs and x64/ARM64 Windows, including cancellation, permission failure, interruption and retained user data.
