# CI and updates

`check.yml` runs on pushes, pull requests and manual dispatch. Rust uses the current
stable toolchain with `fmt`, Clippy (`-D warnings`) and all workspace tests on macOS
and Windows. Both architectures compile the SwiftUI/WinUI application with freshly
generated UniFFI/Interoptopus bindings. Swift helper regressions and C#–Rust integration
regressions run without launching the applications. Pull requests receive no signing
secrets and only a read-only repository token.

The macOS jobs use GitHub's `xcode-27` arm64 image and explicitly select
`latest-stable` through `maxim-lobanov/setup-xcode`; they report Xcode, SDK and Swift
versions in the job log. The [runner software manifest](https://github.com/actions/runner-images/blob/main/images/macos/xcode-27-arm64-Readme.md)
currently includes Xcode 27.0 (27A266a) and macOS SDK 27.0. GitHub still marks the
runner image as preview; selecting `latest-stable` excludes beta Xcode toolchains.
The older `macos-26` images currently top out at Xcode 26.6. Both arm64 and x86_64
apps compile with the selected stable toolchain; x86_64 is cross-compiled on the
arm64 host. Swift/Rust ABI execution runs for arm64, while helper state tests run
on the host. This does not claim Intel runtime validation.

`release.yml` runs for `vMAJOR.MINOR.PATCH` tags or a manually supplied existing tag.
Prerelease tags are rejected and never overwrite stable feeds. It first runs the same
checks against the release tag, then builds and signs both architectures on both
platforms. All eight assets are attached to a draft GitHub Release before publishing
it as latest. A failure leaves the previous stable feeds in place; rerunning a failed
draft is supported. Published releases and downgrades cannot be overwritten.

The Windows package version is the release version with `.0` appended, including
`v0.6.0` → `0.6.0.0`. Zero major is valid for sideloaded MSIX [package identities](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview);
the stricter [Store submission rules](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements)
do not apply to App Installer distribution. macOS uses the release version for the
visible version and the monotonically increasing release workflow run number for
`CFBundleVersion`.

Configure the GitHub `release` environment before publishing. Restrict it to release
tags and trusted maintainers. No certificate, private key, publisher identity or host
is invented by these workflows.

| Type | Name | Value |
| --- | --- | --- |
| Secret | `MACOS_CERTIFICATE_P12` | Base64 Developer ID Application signing certificate with its private key |
| Secret | `MACOS_CERTIFICATE_PASSWORD` | P12 password |
| Variable | `CODE_SIGN_IDENTITY` | Exact Developer ID Application identity |
| Secret | `APPLE_API_PRIVATE_KEY` | App Store Connect API private key, PEM content |
| Variable | `APPLE_API_KEY_ID` | API key ID |
| Variable | `APPLE_API_ISSUER` | Team API issuer UUID |
| Secret | `SPARKLE_PRIVATE_ED_KEY` | Private EdDSA key exported by Sparkle `generate_keys` |
| Variable | `SPARKLE_PUBLIC_ED_KEY` | Matching public EdDSA key |
| Secret | `WINDOWS_CERTIFICATE_PFX` | Base64 trusted code-signing PFX with private key |
| Secret | `WINDOWS_CERTIFICATE_PASSWORD` | PFX password |
| Variable | `WINDOWS_PUBLISHER` | Certificate subject, exactly matching package Publisher |
| Variable | `WINDOWS_TIMESTAMP_URL` | Signing provider's RFC 3161 timestamp service URL |

macOS uses Sparkle 2.10.0's official `generate_appcast`, with a pinned verified
archive SHA-256. The Developer ID application is notarized and stapled before its
final ZIP bytes are EdDSA-signed. Private key files/keychains are deleted after use.
The embedded feed is `https://github.com/<owner>/<repo>/releases/latest/download/appcast-<arch>.xml`;
ZIP enclosures use immutable tag URLs.

Windows creates signed, self-contained MSIX packages and architecture-specific
`BibCiTeX-<arch>.appinstaller` feeds at the same `releases/latest/download` base.
Install through the App Installer file to enroll in system updates. Package name,
publisher and signing identity must remain stable across releases. A self-signed
certificate requires explicit trust on each user machine; production releases should
use a certificate already trusted by Windows. The workflow validates Authenticode
trust and cleans up the imported signing certificate/private key after use.

Branch pushes run checks without publishing a release. Local metadata tests run
with `python3 -m unittest discover -s ci -p 'test_*.py'`; actual notarization,
MSIX signing and release publication require the repository owner's configured
credentials and a separate release trigger.
