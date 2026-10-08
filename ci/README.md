# Build and release automation

The Check workflow builds the Rust workspace and both native applications on x86_64 and ARM64. IDE builds own binding generation. macOS command-line builds do not launch an application. Windows XAML compilation runs on Windows; C# semantic builds on macOS are supplementary.

The Release workflow accepts `vMAJOR.MINOR.PATCH`, `-alpha.N`, and `-beta.N` tags. Publishing a GitHub Release triggers Check and builds both architectures from the resolved tag commit. The workflow validates the full artifact set, uploads it to that existing public Release, and only then advances the separate `update-feed` release. It then publishes the same verified payloads and COS-specific feeds to the primary COS mirror. There is no Tauri migration, MSIX or App Installer output.

## Required configuration

Use a GitHub environment named `release`:

| Type | Name | Purpose |
| --- | --- | --- |
| Secret | `SPARKLE_PRIVATE_ED_KEY` | Exported Sparkle Ed25519 private update key |
| Secret | `COS_SECRET_ID` | CAM publishing subuser SecretId |
| Secret | `COS_SECRET_KEY` | CAM publishing subuser SecretKey |
| Variable | `SPARKLE_PUBLIC_ED_KEY` | Matching public key embedded into the macOS application |

These self-generated keys are free and independent of Apple Developer ID certificates. The workflow needs no commercial signing certificates, Microsoft Store, custom application server. COS hosts the primary update files; GitHub Releases remains the fallback. `GITHUB_TOKEN` provides repository release access; no token is shipped to clients. Keep the same key pair for future Sparkle updates.

The currently configured macOS runner label is `xcode-27`; runners must provide the project's supported Xcode SDK. Windows builds use `windows-2025` and `windows-11-arm`. Release jobs install the pinned Velopack CLI 1.2.161. macOS uses the Sparkle tools that Xcode resolved and verified with SwiftPM, avoiding a second unverified tool download.

## Publishing

Add reviewed release notes in both languages under `release-notes/<version>/`. Tag the exact commit, then publish its GitHub Release (not a draft). A tag push alone does not build release assets. For an already published release, dispatch Release from the maintained branch with its existing tag. The workflow pins all source checkouts to the resolved commit and rechecks the remote tag before uploading; publisher tooling comes from the selected workflow revision. Stable tags require a regular release and alpha/beta tags require the prerelease flag. The workflow serializes publication to keep channel updates ordered. The public Release may initially have no assets. A partial build or failed validation must not advance update feeds. Inspect both the versioned release and `update-feed` after publication. Upload failures can be retried with **Re-run failed jobs**, preserving the original build artifacts and build number. Existing files are skipped only when size and GitHub SHA-256 digest match; conflicting or digest-less files fail before any uploads. Missing files are uploaded without replacement, with `release.json` last. Do not rebuild an already populated version: a new run can produce different package bytes and will correctly refuse to overwrite them. Publication does not edit the release body, prerelease status, or GitHub Latest selection.

The installer build sequence is `github.run_number`; preserve monotonicity if replacing this workflow. The first release using this mechanism is manually installed. Later releases update in place.

## COS primary source and GitHub fallback

The release bucket is `app-release-1302963684` in `ap-guangzhou`. This application's publisher writes only under `bibcitex/`, leaving other apps in the shared bucket untouched. Clients use:

- Primary: `https://app-release-1302963684.cos.ap-guangzhou.myqcloud.com/bibcitex/update-feed`
- Fallback: `https://github.com/tangxiangong/bibcitex/releases/download/update-feed`

Each new check prefers COS. A valid primary feed with no eligible update is not an error and does not query GitHub. On macOS, Sparkle retries feed/transport failures once through the GitHub feed using the original foreground/background check mode; primary errors eligible for retry do not interrupt the user. Cancellation, signature/validation failures and installation errors never trigger a retry. On Windows, the Velopack source retries failed feed requests and package downloads against GitHub. Package fallback retains the original filename, size and SHA-256; it never substitutes a different version. User cancellation stops the operation. Both-source failures remain visible through the existing updater error handling. There is no permanent source preference change.

Enable global acceleration on the bucket: CI uploads through `app-release-1302963684.cos.accelerate.myqcloud.com`; clients download through the normal Guangzhou HTTPS endpoint. Permit anonymous reads of release objects. The publisher explicitly sets `public-read`; its CAM subuser needs `cos:HeadObject`, `cos:PutObject` and `cos:PutObjectACL` on `app-release-1302963684/bibcitex/*` (or the previously configured bucket-wide publishing policy). It does not delete objects, list the bucket or require signed GET access. Secrets may be repository secrets or release-environment secrets; neither is embedded into the app. Storage bucket, region and app prefix are fixed in the publisher, so no GitHub Variables for COS are required.

```text
bibcitex/
  releases/vX.Y.Z/              # Versioned release assets, notes and checksums
  update-feed/
    appcast-<arch>-<channel>-<locale>.xml
    releases.win-<arch>-<channel>.json
    <versioned-full-package>.nupkg
    channels.json
```

The COS publisher validates all four platform/architecture payloads, Sparkle signatures and Windows hashes before network writes. Existing objects are checked using anonymous downloads and SHA-256; different contents fail rather than being overwritten. Uploads retry transient errors and use COS's forbid-overwrite header for immutable objects (the service only enforces that header when bucket versioning has never been enabled). Keep all publishing workflows in the shared `release-feeds` concurrency group; do not run independent writers concurrently. Initial catalogs and mutable feed pointers use `no-cache, max-age=60`; versioned assets use `public, max-age=31536000, immutable`. All payloads are uploaded and publicly verified before channel feeds advance. `channels.json` reserves the highest target version for each audience before client-facing feeds change; equal-version retries regenerate pointers so interrupted runs can finish without allowing an older repair to downgrade partially published feeds. Do not delete the catalog or configure automatic expiry/archival for referenced packages. Each source maintains its own monotonically advancing stable/beta/alpha audiences; failure halfway through feed upload can temporarily leave audiences on different valid versions, and retry completes publication.

The COS appcasts point to COS archives and localized notes; GitHub appcasts retain GitHub links. Velopack packages live beside each source's channel feeds and have identical bytes on both hosts. Publishing GitHub first ensures the fallback is ready before COS advances. If COS publication fails, GitHub remains valid; rerun the failed publish job, or use **Publish existing release to COS** with the existing tag. That manual workflow downloads and verifies already published GitHub assets without rebuilding or changing their signatures. Do not rebuild a populated release to repair a mirror.

Verification: `just check-rust`, `just test-ci`, macOS compilation, and non-UI C#/Rust transport/integration checks. GUI update acceptance, real CAM permissions, global acceleration and signed in-place upgrades still require validation against a published release. Compilation or a local simulated transport test is not a live deployment test.

## Automatic Homebrew tap updates

After **Release** completes successfully, **Update Homebrew tap** runs through a
`workflow_run` completion event. No scheduled polling is used. It downloads and
verifies both macOS archives from the newest complete stable release, then
commits the version and SHA-256 changes to `tangxiangong/homebrew-tap`.
The tap updater ignores prereleases and incomplete releases and prevents
version downgrades. Its tests and commit helper live in the tap repository.

Configure repository secret `HOMEBREW_TAP_DEPLOY_KEY` with the private SSH key
whose public key is installed as a write-enabled deploy key on the tap only.
This grants no write access to other repositories. The completion workflow uses
the maintained tap `main` branch, never code or artifacts from the triggering
workflow. The workflow file must exist on this repository's default branch.
For recovery after a tap update failure, manually dispatch **Update Homebrew tap**;
there is no need to rebuild or republish release assets.

## Linux releases

Linux uses native Ubuntu 24.04 x86-64 and ARM64 runners with GPUI Kit. The release
job builds a portable `.tar.gz` and a `.deb`, plus `linux-ARCH.json` and its detached
Ed25519 signature. `xtask package-linux` generates packages without launching the
app; `xtask sign-linux` signs metadata; release verification validates both Linux
architectures and both payload hashes whenever Linux assets are present.

The `release` environment must provide:

- Variable `BIBCITEX_LINUX_UPDATE_PUBLIC_KEY`: base64 32-byte Ed25519 public key,
  embedded in the Linux release binary and used by publication verification.
- Secret `LINUX_UPDATE_PRIVATE_KEY`: base64 32-byte private seed, available only
  to the metadata-signing step. The signer verifies that it matches the public key.

Linux update selection uses the existing COS channel catalog, immutable versioned
artifacts, and GitHub fallback. No Linux-specific mutable feed is required. The
standalone COS repair workflow also receives the Linux verification public key.
Production signing and delivery still require configured credentials and a release;
unit tests use explicit fixture keys and never create production credentials.
