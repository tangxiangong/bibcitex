# Build and release automation

The Check workflow builds the Rust workspace and both native applications on x86_64 and ARM64. IDE builds own binding generation. macOS command-line builds do not launch an application. Windows XAML compilation runs on Windows; C# semantic builds on macOS are supplementary.

The Release workflow accepts `vMAJOR.MINOR.PATCH`, `-alpha.N`, and `-beta.N` tags. Publishing a GitHub Release triggers Check and builds from the resolved tag commit: one macOS Universal 2 app for Apple Silicon and Intel, plus separate Windows x64 and ARM64 packages. The workflow validates the full artifact set, uploads it to that existing public Release, and only then advances the separate `update-feed` release. It then publishes the same verified payloads and COS-specific feeds to the primary COS mirror. There is no Tauri migration, MSIX or App Installer output.

## Required configuration

Use a GitHub environment named `release`:

| Type | Name | Purpose |
| --- | --- | --- |
| Secret | `SPARKLE_PRIVATE_ED_KEY` | Exported Sparkle Ed25519 private update key |
| Secret | `WINDOWS_UPDATE_PRIVATE_KEY` | Base64 DER PKCS#8 ECDSA P-256 private update key |
| Secret | `COS_SECRET_ID` | CAM publishing subuser SecretId |
| Secret | `COS_SECRET_KEY` | CAM publishing subuser SecretKey |
| Variable | `SPARKLE_PUBLIC_ED_KEY` | Matching public key embedded into the macOS application |
| Variable | `WINDOWS_UPDATE_PUBLIC_KEY` | Matching base64 DER SPKI P-256 public key embedded into Windows builds |

These self-generated keys are free and independent of Apple Developer ID certificates. The workflow needs no commercial signing certificates, Microsoft Store, custom application server. COS hosts the primary update files; GitHub Releases remains the fallback. `GITHUB_TOKEN` provides repository release access; no token is shipped to clients. Keep each platform’s key pair for future updates. The Windows key is separate from the Sparkle key and from Windows executable signing. Generate a P-256 pair with .NET `ECDsa.Create(ECCurve.NamedCurves.nistP256)` or OpenSSL; store the base64 DER `ExportPkcs8PrivateKey()` as the secret and `ExportSubjectPublicKeyInfo()` as the variable. Never commit a private key. A key change prevents existing signed clients from verifying subsequent feeds; plan a migration before replacing it.

The currently configured macOS runner label is `xcode-27`; runners must provide the project's supported Xcode SDK. Windows builds use `windows-2025` and `windows-11-arm`. Release jobs install the pinned Velopack CLI 1.2.161. macOS uses the Sparkle tools that Xcode resolved and verified with SwiftPM, avoiding a second unverified tool download.

## Publishing

Add reviewed release notes in both languages under `release-notes/<version>/`. Follow the [platform visibility rules](../release-notes/README.md) for common and platform-specific announcements; publication generates filtered Markdown and Windows packaging uses the same selector. Tag the exact commit, then publish its GitHub Release (not a draft). A tag push alone does not build release assets. For an already published release, dispatch Release from the maintained branch with its existing tag. The workflow pins all source checkouts to the resolved commit and rechecks the remote tag before uploading; publisher tooling comes from the selected workflow revision. Stable tags require a regular release and alpha/beta tags require the prerelease flag. The workflow serializes publication to keep channel updates ordered. The public Release may initially have no assets. A partial build or failed validation must not advance update feeds. Inspect both the versioned release and `update-feed` after publication. Upload failures can be retried with **Re-run failed jobs**, preserving the original build artifacts and build number. Existing files are skipped only when size and GitHub SHA-256 digest match; conflicting or digest-less files fail before any uploads. Missing files are uploaded without replacement, with `release.json` last. Do not rebuild an already populated version: a new run can produce different package bytes and will correctly refuse to overwrite them. Publication does not edit the release body, prerelease status, or GitHub Latest selection.

Before publication, `xtask sign-windows <assets>` requires both Windows keys, proves they match, validates both architectures and creates `releases.win-<arch>-<channel>.signed.json`. Publication verifies each signature and requires its assets to match the ordinary Velopack feed before any upload. Every promoted stable/beta/alpha audience receives both legacy JSON and the signed envelope; COS mirrors identical signed bytes. New clients never downgrade to an unsigned feed. Existing releases without signed Windows metadata are rejected by this publisher rather than silently treated as authenticated. Do not repair an old populated release by rebuilding it; prepare its migration explicitly.

The installer build sequence is `github.run_number`; preserve monotonicity if replacing this workflow. The first release using this mechanism is manually installed. Later releases update in place.

macOS produces `BibCiTeX-<version>-macos-universal.dmg`,
`BibCiTeX-<version>-macos-universal.app.zip` and `appcast-universal.xml`.
The release script builds both Rust targets, then verifies that every Mach-O binary
in the app, including embedded Sparkle code, contains both `arm64` and `x86_64`
before packaging. Publication verifies the archive's Sparkle signature and generates
the existing `appcast-arm64-<channel>-<locale>.xml` and
`appcast-x86_64-<channel>-<locale>.xml` names from that single signed appcast.
Both architectures receive the same Universal archive; keep these feed names on
both hosts because existing clients and Universal builds continue requesting them.
Preserve the app's bundle identifier and Sparkle key, and increase the semantic
version and build number so installed releases can update directly.

The publisher also accepts complete legacy macOS releases containing both
architecture-specific archives and appcasts. Their signatures and payload URLs
remain architecture-specific, so mirroring or retrying an older release does not
require rebuilding it. An incomplete Universal release fails validation rather
than falling back to legacy packages.

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

The COS publisher validates the macOS Universal archive (or both legacy architecture archives), Sparkle signatures and both Windows payloads before network writes. Existing objects are checked using anonymous downloads and SHA-256; different contents fail rather than being overwritten. Uploads retry transient errors and use COS's forbid-overwrite header for immutable objects (the service only enforces that header when bucket versioning has never been enabled). Keep all publishing workflows in the shared `release-feeds` concurrency group; do not run independent writers concurrently. Initial catalogs and mutable feed pointers use `no-cache, max-age=60`; versioned assets use `public, max-age=31536000, immutable`. All payloads are uploaded and publicly verified before channel feeds advance. `channels.json` reserves the highest target version for each audience before client-facing feeds change; equal-version retries regenerate pointers so interrupted runs can finish without allowing an older repair to downgrade partially published feeds. Do not delete the catalog or configure automatic expiry/archival for referenced packages. Each source maintains its own monotonically advancing stable/beta/alpha audiences; failure halfway through feed upload can temporarily leave audiences on different valid versions, and retry completes publication.

The COS appcasts point to COS archives and localized notes; GitHub appcasts retain GitHub links. Velopack packages live beside each source's channel feeds and have identical bytes on both hosts. Publishing GitHub first ensures the fallback is ready before COS advances. If COS publication fails, GitHub remains valid; rerun the failed publish job, or use **Publish existing release to COS** with the existing tag. That manual workflow downloads and verifies already published GitHub assets without rebuilding or changing their signatures. Do not rebuild a populated release to repair a mirror.

If a maintainer has explicitly rebuilt and replaced a version's GitHub assets,
dispatch **Publish existing release to COS** with that tag and
`replace_existing=true`. This opt-in invokes `xtask replace-cos-release`: it
requires an existing valid COS catalog, validates the complete signed release,
replaces only that version's COS assets and matching Windows feed packages, then
refreshes eligible channel pointers. Other versions remain intact and older
repairs cannot downgrade channels. Ordinary mirroring defaults to
`replace_existing=false` and continues to reject conflicting bytes. The manual
workflow reads GitHub assets; it does not rebuild or replace GitHub files itself.

Verification: `just check-rust`, `just test-ci`, macOS compilation, and non-UI C#/Rust transport/integration checks. GUI update acceptance, real CAM permissions, global acceleration and signed in-place upgrades still require validation against a published release. Compilation or a local simulated transport test is not a live deployment test.

## Automatic Homebrew tap updates

After **Release** completes successfully, **Update Homebrew tap** runs through a
`workflow_run` completion event. No scheduled polling is used. It downloads and
verifies the macOS Universal archive from the newest complete stable release,
then commits the version, URL and SHA-256 changes to `tangxiangong/homebrew-tap`.
The tap updater migrates the cask from architecture-specific URLs/checksums to a
single Universal URL/checksum when that asset becomes available, and continues
to support older releases with separate archives. Deploy the tap updater change
to the tap's `main` branch before publishing the first Universal release.
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

Linux Release CI is temporarily disabled until the client is ready for release.
The Release workflow explicitly emits `linux=false`; restore platform detection there
only after Linux release acceptance. Release passes that switch to the reusable Check
workflow, so Linux client checks cannot block macOS/Windows publication while disabled.
Ordinary Check CI, including Linux client builds and tests, remains enabled. While Release CI is disabled, publication collects only
macOS and Windows artifacts, excludes Linux announcement assets, and does not build,
sign, upload, or publish Linux packages to GitHub or COS.

When re-enabled, Linux uses native Ubuntu 24.04 x86-64 and ARM64 runners with GPUI Kit. The release
job builds a portable `.tar.gz` and a `.deb`, plus `linux-ARCH.json` and its detached
Ed25519 signature. `xtask package-linux` generates packages without launching the
app; `xtask sign-linux` signs metadata; release verification validates both Linux
architectures and both payload hashes whenever Linux assets are present.

Before re-enabling Linux releases, the `release` environment must provide:

- Variable `BIBCITEX_LINUX_UPDATE_PUBLIC_KEY`: base64 32-byte Ed25519 public key,
  embedded in the Linux release binary and used by publication verification.
- Secret `LINUX_UPDATE_PRIVATE_KEY`: base64 32-byte private seed, available only
  to the metadata-signing step. The signer verifies that it matches the public key.

Linux update selection uses the existing COS channel catalog, immutable versioned
artifacts, and GitHub fallback. No Linux-specific mutable feed is required. The
standalone COS repair workflow also receives the Linux verification public key.
Production signing and delivery still require configured credentials and a release;
unit tests use explicit fixture keys and never create production credentials.
