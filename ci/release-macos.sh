#!/bin/bash
# Only the release workflow invokes this; never launches the application.
set -euo pipefail
required=(MACOS_CERTIFICATE_P12 MACOS_CERTIFICATE_PASSWORD CODE_SIGN_IDENTITY APPLE_API_PRIVATE_KEY APPLE_API_KEY_ID APPLE_API_ISSUER SPARKLE_PRIVATE_ED_KEY SPARKLE_PUBLIC_ED_KEY APP_VERSION BUILD_NUMBER ARCH RELEASE_TAG GITHUB_REPOSITORY RUNNER_TEMP)
for name in "${required[@]}"; do
    [[ -n "${!name:-}" ]] || { echo "::error::Missing release configuration: $name" >&2; exit 1; }
done
root="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d "$RUNNER_TEMP/bibcitex-sign.XXXXXX")"
keychain="$work/signing.keychain-db"
keychain_password="$(openssl rand -hex 32)"
cleanup() {
    security delete-keychain "$keychain" >/dev/null 2>&1 || true
    rm -rf "$work"
}
trap cleanup EXIT
umask 077
printf '%s' "$MACOS_CERTIFICATE_P12" | base64 --decode > "$work/signing.p12"
printf '%s' "$APPLE_API_PRIVATE_KEY" > "$work/AuthKey.p8"
printf '%s' "$SPARKLE_PRIVATE_ED_KEY" > "$work/sparkle.key"
security create-keychain -p "$keychain_password" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$keychain_password" "$keychain"
security import "$work/signing.p12" -k "$keychain" -P "$MACOS_CERTIFICATE_PASSWORD" -T /usr/bin/codesign >/dev/null
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$keychain_password" "$keychain" >/dev/null
security list-keychains -d user -s "$keychain" "$HOME/Library/Keychains/login.keychain-db"
export SPARKLE_FEED_URL="https://github.com/$GITHUB_REPOSITORY/releases/latest/download/appcast-$ARCH.xml"
export CONFIGURATION=release
unset BUILD_ONLY
bash "$root/macos/build.sh"
app="$root/dist/macos/$ARCH/BibCiTeX.app"
codesign --verify --deep --strict --verbose=2 "$app"
out="$root/dist/release/macos-$ARCH"
mkdir -p "$out"
archive="$out/BibCiTeX-$APP_VERSION-macos-$ARCH.zip"
ditto -c -k --keepParent "$app" "$archive"
xcrun notarytool submit "$archive" --key "$work/AuthKey.p8" --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER" --wait
xcrun stapler staple "$app"
xcrun stapler validate "$app"
# Notarization/stapling changes the archive; sign the final bytes only.
rm "$archive"
ditto -c -k --keepParent "$app" "$archive"
curl --fail --location --retry 3 https://github.com/sparkle-project/Sparkle/releases/download/2.10.0/Sparkle-2.10.0.tar.xz --output "$work/Sparkle.tar.xz"
printf 'c2bf58aa8387266ac179357b1415d6f2635f044da8be41042af32425dae6da0c  %s\n' "$work/Sparkle.tar.xz" | shasum -a 256 --check
mkdir "$work/sparkle"
tar -xf "$work/Sparkle.tar.xz" -C "$work/sparkle"
"$work/sparkle/bin/generate_appcast" --ed-key-file "$work/sparkle.key" --maximum-deltas 0 \
    --download-url-prefix "https://github.com/$GITHUB_REPOSITORY/releases/download/$RELEASE_TAG/" \
    -o "$out/appcast-$ARCH.xml" "$out"
python3 "$root/ci/verify_release.py" macos "$out" "$APP_VERSION" "$GITHUB_REPOSITORY" "$RELEASE_TAG"
