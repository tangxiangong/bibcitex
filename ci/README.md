# Build and release automation

The Check workflow builds the Rust workspace and both native applications on x86_64 and ARM64. IDE builds own binding generation. macOS command-line builds do not launch an application. Windows XAML compilation runs on Windows; C# semantic builds on macOS are supplementary.

The Release workflow accepts `vMAJOR.MINOR.PATCH`, `-alpha.N`, and `-beta.N` tags. It runs Check, builds both architectures, validates the full artifact set, publishes a versioned GitHub Release and then advances the separate `update-feed` release. There is no Tauri migration, MSIX or App Installer output.

## Required configuration

Use a GitHub environment named `release`:

| Type | Name | Purpose |
| --- | --- | --- |
| Secret | `SPARKLE_PRIVATE_ED_KEY` | Exported Sparkle Ed25519 private update key |
| Variable | `SPARKLE_PUBLIC_ED_KEY` | Matching public key embedded into the macOS application |

These self-generated keys are free and independent of Apple Developer ID certificates. The workflow needs no commercial signing certificates, Microsoft Store, custom server or cloud storage. `GITHUB_TOKEN` provides repository release access; no token is shipped to clients. Keep the same key pair for future Sparkle updates.

The currently configured macOS runner label is `xcode-27`; runners must provide the project's supported Xcode SDK. Windows builds use `windows-2025` and `windows-11-arm`. Release jobs install the pinned Velopack CLI 1.2.161. macOS uses the Sparkle tools that Xcode resolved and verified with SwiftPM, avoiding a second unverified tool download.

## Publishing

Add reviewed release notes in both languages under `release-notes/<version>/`. Tag the exact commit, then push the tag or dispatch Release against that tag. The workflow serializes publication to keep channel updates ordered. A partial build or failed validation must not publish an installable version. Inspect both the versioned release and `update-feed` after publication.

The installer build sequence is `github.run_number`; preserve monotonicity if replacing this workflow. The first release using this mechanism is manually installed. Later releases update in place.

See [the update protocol](../docs/updates.md) for asset names, channel rules, verification and acceptance tests.
