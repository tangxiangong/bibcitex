#!/bin/bash
# User-invoked only. This script builds and bundles; it does not launch the app.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
configuration="${CONFIGURATION:-release}"
[[ "$configuration" == release || "$configuration" == debug ]] || { echo "CONFIGURATION must be debug or release" >&2; exit 1; }
architecture="${ARCH:-$(uname -m)}"
case "$architecture" in arm64) rust_target=aarch64-apple-darwin;; x86_64) rust_target=x86_64-apple-darwin;; *) echo "Unsupported ARCH" >&2; exit 1;; esac
rust_args=(build --manifest-path "$root/Cargo.toml" -p bibcitex-ffi --target "$rust_target")
[[ "$configuration" == release ]] && rust_args+=(--release)
cargo "${rust_args[@]}"
rust_dir="$root/target/$rust_target/$configuration"
package="$root/macos/BibCiTeX"
bash "$root/bindings/generate-swift.sh" "$rust_dir/libbibcitex_ffi.dylib"
generated="$root/bindings/generated/swift"
mkdir -p "$package/Generated/BibCiTeXCore" "$package/Generated/BibCiTeXCoreFFI"
cp "$generated/BibCiTeXCore.swift" "$package/Generated/BibCiTeXCore/"
cp "$generated/BibCiTeXCoreFFI.h" "$package/Generated/BibCiTeXCoreFFI/"
cp "$generated/BibCiTeXCoreFFI.modulemap" "$package/Generated/BibCiTeXCoreFFI/module.modulemap"
scratch="$root/target/swift/$architecture"
swift_args=(--package-path "$package" --scratch-path "$scratch" --build-system xcode -c "$configuration" --arch "$architecture")
swift build "${swift_args[@]}" -Xlinker "$rust_dir/libbibcitex_ffi.a" -Xlinker -L -Xlinker "$rust_dir" -Xlinker -rpath -Xlinker @executable_path/../Frameworks
products="$(swift build "${swift_args[@]}" --show-bin-path)"
# Xcode's SwiftPM accessor resolves Bundle.main.resourceURL; refuse a build-system
# change that would silently leave release bundles looking in the build directory.
python3 - "$scratch" <<'PYRESOURCE'
from pathlib import Path
import sys
accessors = list(Path(sys.argv[1]).rglob("resource_bundle_accessor.swift"))
if not accessors or any("Bundle.main.resourceURL" not in p.read_text() for p in accessors):
    raise SystemExit("SwiftPM resource accessor does not support Contents/Resources; bundle layout must be reviewed")
PYRESOURCE
if [[ "${BUILD_ONLY:-0}" == 1 ]]; then
    echo "$products/BibCiTeX"
    exit 0
fi
app="$root/dist/macos/$architecture/BibCiTeX.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$app/Contents/Frameworks"
cp "$products/BibCiTeX" "$app/Contents/MacOS/BibCiTeX"
cp "$root/macos/Info.plist" "$app/Contents/Info.plist"
if [[ -n "${APP_VERSION:-}" ]]; then
    [[ "$APP_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "APP_VERSION must have three numeric components" >&2; exit 1; }
    /usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $APP_VERSION" "$app/Contents/Info.plist"
fi
if [[ -n "${BUILD_NUMBER:-}" ]]; then
    [[ "$BUILD_NUMBER" =~ ^[0-9]+$ ]] || { echo "BUILD_NUMBER must be numeric" >&2; exit 1; }
    /usr/libexec/PlistBuddy -c "Set :CFBundleVersion $BUILD_NUMBER" "$app/Contents/Info.plist"
fi
cp "$root/assets/app-icons/icon.icns" "$app/Contents/Resources/icon.icns"
for bundle in "$products"/*.bundle; do
    [[ -d "$bundle" ]] && ditto "$bundle" "$app/Contents/Resources/$(basename "$bundle")"
done
sparkle="$(find "$scratch/artifacts" -path '*/macos-arm64_x86_64/Sparkle.framework' -type d -print -quit)"
[[ -n "$sparkle" ]] || { echo "Sparkle.framework not found" >&2; exit 1; }
ditto "$sparkle" "$app/Contents/Frameworks/Sparkle.framework"
# Release configuration is injected by the publisher, never fabricated in source.
if [[ -n "${SPARKLE_FEED_URL:-}" && -n "${SPARKLE_PUBLIC_ED_KEY:-}" ]]; then
    /usr/libexec/PlistBuddy -c "Add :SUFeedURL string $SPARKLE_FEED_URL" "$app/Contents/Info.plist"
    /usr/libexec/PlistBuddy -c "Add :SUPublicEDKey string $SPARKLE_PUBLIC_ED_KEY" "$app/Contents/Info.plist"
fi
identity="${CODE_SIGN_IDENTITY:--}"
# Sign nested Sparkle code inside-out before signing the application.
find "$app/Contents/Frameworks/Sparkle.framework" -type f -name Autoupdate -print0 | while IFS= read -r -d '' tool; do
    codesign --force --options runtime --sign "$identity" "$tool"
done
find "$app/Contents/Frameworks/Sparkle.framework" -type d \( -name '*.xpc' -o -name '*.app' \) -depth -print0 | while IFS= read -r -d '' nested; do
    if [[ "$nested" == */Downloader.xpc ]]; then
        # Sparkle >= 2.6 requires preserving any custom Downloader sandbox entitlements.
        codesign --force --options runtime --preserve-metadata=entitlements --sign "$identity" "$nested"
    else
        codesign --force --options runtime --sign "$identity" "$nested"
    fi
done
codesign --force --options runtime --sign "$identity" "$app/Contents/Frameworks/Sparkle.framework"
codesign --force --options runtime --sign "$identity" "$app"
echo "$app"
