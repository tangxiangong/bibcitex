#!/bin/bash
# CLI/Just uses the same checked-in Xcode project as the IDE. Never launches the app.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
case "${CONFIGURATION:-release}" in
    Debug|debug) configuration=Debug;;
    Release|release) configuration=Release;;
    *) echo "CONFIGURATION must be debug or release" >&2; exit 1;;
esac
architecture="${ARCH:-$(uname -m)}"
case "$architecture" in
    arm64|x86_64) architectures="$architecture";;
    universal) architectures='arm64 x86_64';;
    *) echo "ARCH must be arm64, x86_64 or universal" >&2; exit 1;;
esac
derived="${DERIVED_DATA_PATH:-$root/target/xcode/$architecture}"
build_args=(-project "$root/macos/BibCiTeX.xcodeproj" -scheme BibCiTeX -configuration "$configuration"
    -derivedDataPath "$derived" -destination 'generic/platform=macOS'
    "ARCHS=$architectures" ONLY_ACTIVE_ARCH=NO)
# Keep certificate-free CI builds available, but honor requested signing for
# local runs: linkd validates the app's signed team identity.
identity="${CODE_SIGN_IDENTITY:--}"
if [[ -n "${DEVELOPMENT_TEAM:-}" ]]; then
    identity="${CODE_SIGN_IDENTITY:-Apple Development}"
    build_args+=(CODE_SIGNING_ALLOWED=YES "DEVELOPMENT_TEAM=$DEVELOPMENT_TEAM"
        "CODE_SIGN_IDENTITY=$identity" "CODE_SIGN_STYLE=${CODE_SIGN_STYLE:-Automatic}")
elif [[ "$identity" != - ]]; then
    build_args+=(CODE_SIGNING_ALLOWED=YES "CODE_SIGN_IDENTITY=$identity" CODE_SIGN_STYLE=Manual)
else
    build_args+=(CODE_SIGNING_ALLOWED=NO)
fi
if [[ -n "${APP_VERSION:-}" ]]; then
    [[ "$APP_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "APP_VERSION must have three numeric components" >&2; exit 1; }
    build_args+=("MARKETING_VERSION=$APP_VERSION")
fi
if [[ -n "${BUILD_NUMBER:-}" ]]; then
    [[ "$BUILD_NUMBER" =~ ^[0-9]+$ ]] || { echo "BUILD_NUMBER must be numeric" >&2; exit 1; }
    build_args+=("CURRENT_PROJECT_VERSION=$BUILD_NUMBER")
fi
xcodebuild "${build_args[@]}" build
product="$derived/Build/Products/$configuration/BibCiTeX.app"
[[ -x "$product/Contents/MacOS/BibCiTeX" ]] || { echo "Xcode did not produce BibCiTeX.app" >&2; exit 1; }
[[ -d "$product/Contents/Resources/MathJaxSwift_MathJaxSwift.bundle" ]] || { echo "MathJax resource bundle is missing" >&2; exit 1; }
[[ -f "$product/Contents/Resources/library.svg" ]] || { echo "SVG resources are missing" >&2; exit 1; }
[[ -d "$product/Contents/Frameworks/Sparkle.framework" ]] || { echo "Sparkle framework is missing" >&2; exit 1; }
if [[ "${BUILD_ONLY:-0}" == 1 ]]; then
    echo "$product/Contents/MacOS/BibCiTeX"
    exit 0
fi
app="$root/dist/macos/$architecture/BibCiTeX.app"
mkdir -p "$(dirname "$app")"
# This destination contains only the generated app, never source or user libraries.
rm -rf "$app"
ditto "$product" "$app"
if [[ -n "${SPARKLE_FEED_URL:-}" && -n "${SPARKLE_PUBLIC_ED_KEY:-}" ]]; then
    /usr/libexec/PlistBuddy -c "Add :SUFeedURL string $SPARKLE_FEED_URL" "$app/Contents/Info.plist"
    /usr/libexec/PlistBuddy -c "Add :SUPublicEDKey string $SPARKLE_PUBLIC_ED_KEY" "$app/Contents/Info.plist"
fi
# Sparkle's nested code is signed inside-out, preserving Downloader entitlements.
find "$app/Contents/Frameworks/Sparkle.framework" -type f -name Autoupdate -print0 | while IFS= read -r -d '' tool; do
    codesign --force --options runtime --sign "$identity" "$tool"
done
find "$app/Contents/Frameworks/Sparkle.framework" -type d \( -name '*.xpc' -o -name '*.app' \) -depth -print0 | while IFS= read -r -d '' nested; do
    if [[ "$nested" == */Downloader.xpc ]]; then
        codesign --force --options runtime --preserve-metadata=entitlements --sign "$identity" "$nested"
    else
        codesign --force --options runtime --sign "$identity" "$nested"
    fi
done
codesign --force --options runtime --sign "$identity" "$app/Contents/Frameworks/Sparkle.framework"
codesign --force --options runtime --sign "$identity" "$app"
echo "$app"
