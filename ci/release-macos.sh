#!/bin/bash
# Invoked by the release workflow; no commercial certificate or GUI launch required.
set -euo pipefail
for name in SPARKLE_PRIVATE_ED_KEY SPARKLE_PUBLIC_ED_KEY APP_VERSION BUILD_NUMBER ARCH RELEASE_TAG GITHUB_REPOSITORY RUNNER_TEMP; do
    [[ -n "${!name:-}" ]] || { echo "::error::Missing release configuration: $name" >&2; exit 1; }
done
root="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d "$RUNNER_TEMP/bibcitex-release.XXXXXX")"
trap 'rm -rf "$work"' EXIT
umask 077
printf '%s' "$SPARKLE_PRIVATE_ED_KEY" > "$work/sparkle.key"
export UPDATE_BASE_URL="https://github.com/$GITHUB_REPOSITORY/releases/download/update-feed"
export CONFIGURATION=release BUILD_ONLY=0 CODE_SIGN_IDENTITY=-
bash "$root/macos/build.sh"
app="$root/dist/macos/$ARCH/BibCiTeX.app"
out="$root/dist/release/macos-$ARCH"
mkdir -p "$out" "$work/archives" "$work/dmg"
archive="BibCiTeX-$APP_VERSION-macos-$ARCH.app.zip"
ditto -c -k --keepParent "$app" "$work/archives/$archive"
# Use the exact Sparkle tools resolved by Xcode and verified by SwiftPM's binary checksum.
sparkle="${DERIVED_DATA_PATH:-$root/target/xcode/$ARCH}/SourcePackages/artifacts/sparkle/Sparkle/bin"
"$sparkle/generate_appcast" --ed-key-file "$work/sparkle.key" --maximum-deltas 0 \
    --download-url-prefix "https://github.com/$GITHUB_REPOSITORY/releases/download/$RELEASE_TAG/" \
    -o "$out/appcast-$ARCH.xml" "$work/archives"
cp "$work/archives/$archive" "$out/$archive"
ditto "$app" "$work/dmg/BibCiTeX.app"
ln -s /Applications "$work/dmg/Applications"
hdiutil create -quiet -volname BibCiTeX -srcfolder "$work/dmg" -format UDZO "$out/BibCiTeX-$APP_VERSION-macos-$ARCH.dmg"
