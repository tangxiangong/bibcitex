# Build and release automation

The Check workflow builds the Rust workspace and both native applications on x86_64 and ARM64. IDE builds own binding generation. macOS command-line builds do not launch an application. Windows XAML compilation runs on Windows; C# semantic builds on macOS are supplementary.

The Release workflow accepts `vMAJOR.MINOR.PATCH`, `-alpha.N`, and `-beta.N` tags. Publishing a GitHub Release triggers Check and builds both architectures from the resolved tag commit. The workflow validates the full artifact set, uploads it to that existing public Release, and only then advances the separate `update-feed` release. There is no Tauri migration, MSIX or App Installer output.

## Required configuration

Use a GitHub environment named `release`:

| Type | Name | Purpose |
| --- | --- | --- |
| Secret | `SPARKLE_PRIVATE_ED_KEY` | Exported Sparkle Ed25519 private update key |
| Variable | `SPARKLE_PUBLIC_ED_KEY` | Matching public key embedded into the macOS application |

These self-generated keys are free and independent of Apple Developer ID certificates. The workflow needs no commercial signing certificates, Microsoft Store, custom server or cloud storage. `GITHUB_TOKEN` provides repository release access; no token is shipped to clients. Keep the same key pair for future Sparkle updates.

The currently configured macOS runner label is `xcode-27`; runners must provide the project's supported Xcode SDK. Windows builds use `windows-2025` and `windows-11-arm`. Release jobs install the pinned Velopack CLI 1.2.161. macOS uses the Sparkle tools that Xcode resolved and verified with SwiftPM, avoiding a second unverified tool download.

## Publishing

Add reviewed release notes in both languages under `release-notes/<version>/`. Tag the exact commit, then publish its GitHub Release (not a draft). A tag push alone does not build release assets. For an already published release, dispatch Release from the maintained branch with its existing tag. The workflow pins all source checkouts to the resolved commit and rechecks the remote tag before uploading; publisher tooling comes from the selected workflow revision. Stable tags require a regular release and alpha/beta tags require the prerelease flag. The workflow serializes publication to keep channel updates ordered. The public Release may initially have no assets. A partial build or failed validation must not advance update feeds. Inspect both the versioned release and `update-feed` after publication. Upload failures can be retried with **Re-run failed jobs**, preserving the original build artifacts and build number. Existing files are skipped only when size and GitHub SHA-256 digest match; conflicting or digest-less files fail before any uploads. Missing files are uploaded without replacement, with `release.json` last. Do not rebuild an already populated version: a new run can produce different package bytes and will correctly refuse to overwrite them. Publication does not edit the release body, prerelease status, or GitHub Latest selection.

The installer build sequence is `github.run_number`; preserve monotonicity if replacing this workflow. The first release using this mechanism is manually installed. Later releases update in place.

See [the update protocol](../docs/updates.md) for asset names, channel rules, verification and acceptance tests.
