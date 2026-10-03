#!/bin/bash
# Xcode's RustBindings target invokes this before compiling the generated Swift module.
set -euo pipefail
repo="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo"
if command -v cargo >/dev/null 2>&1; then
    cargo_bin="$(command -v cargo)"
elif [[ -x "$HOME/.cargo/bin/cargo" ]]; then
    cargo_bin="$HOME/.cargo/bin/cargo"
else
    echo "error: Rust is required. Install rustup from https://rustup.rs, then build again." >&2
    exit 1
fi
export PATH="$(dirname "$cargo_bin"):$PATH"
case "${CONFIGURATION:-Debug}" in
    Debug|debug) profile=debug;;
    Release|release) profile=release;;
    *) echo "error: Unsupported configuration: $CONFIGURATION" >&2; exit 1;;
esac
output="${BIBCITEX_BINDINGS_DIR:?Xcode must provide BIBCITEX_BINDINGS_DIR}"
rust_build="${BIBCITEX_RUST_TARGET_DIR:-$output/rust-target}"
architectures="${ARCHS:?Xcode must provide its effective ARCHS}"
echo "Building Rust bindings for: $architectures ($profile)"
mkdir -p "$output/include" "$output/raw"
# XCBuild reads the discovered Rust inputs on subsequent builds, including newly
# imported modules, so unchanged UI builds do not rerun Cargo or UniFFI.
escape_dependency() {
    local value="$1"
    value="${value//\\/\\\\}"
    value="${value// /\\ }"
    value="${value//#/\\#}"
    value="${value//\$/\$\$}"
    printf '%s' "$value"
}
{
    escape_dependency "$output/BibCiTeXCore.swift"
    printf ':'
    while IFS= read -r -d '' input; do
        printf ' '
        escape_dependency "$input"
    done < <(find "$repo/crates/bibcitex-core" "$repo/crates/bibcitex-service" "$repo/crates/bibcitex-ffi" "$repo/crates/xpaste" "$repo/bindings/generator" -type f \( -name '*.rs' -o -name '*.toml' \) -print0)
    printf '\n'
} > "$output/rust-bindings.d"
archives=()
metadata_library=""
for architecture in $architectures; do
    case "$architecture" in
        arm64) rust_target=aarch64-apple-darwin;;
        x86_64) rust_target=x86_64-apple-darwin;;
        *) echo "error: Unsupported macOS architecture: $architecture" >&2; exit 1;;
    esac
    if command -v rustup >/dev/null 2>&1 && ! rustup target list --installed | grep -Fxq "$rust_target"; then
        rustup target add "$rust_target"
    fi
    cargo_args=(build --locked --manifest-path "$repo/Cargo.toml" --target-dir "$rust_build" -p bibcitex-ffi --target "$rust_target")
    if [[ "$profile" == release ]]; then cargo_args+=(--release); fi
    "$cargo_bin" "${cargo_args[@]}"
    archive="$rust_build/$rust_target/$profile/libbibcitex_ffi.a"
    archives+=("$archive")
    metadata_library="$rust_build/$rust_target/$profile/libbibcitex_ffi.dylib"
done
# Generate using the official UniFFI CLI; no hand-maintained Swift/C declarations.
CARGO_TARGET_DIR="$rust_build" bash "$repo/bindings/generate-swift.sh" "$metadata_library" "$output/raw"
copy_changed() {
    if ! cmp -s "$1" "$2"; then cp "$1" "$2"; fi
}
copy_changed "$output/raw/BibCiTeXCore.swift" "$output/BibCiTeXCore.swift"
copy_changed "$output/raw/BibCiTeXCoreFFI.h" "$output/include/BibCiTeXCoreFFI.h"
copy_changed "$output/raw/BibCiTeXCoreFFI.modulemap" "$output/include/module.modulemap"
if [[ ${#archives[@]} -eq 1 ]]; then
    copy_changed "${archives[0]}" "$output/libbibcitex_ffi.a"
else
    xcrun lipo -create "${archives[@]}" -output "$output/libbibcitex_ffi.universal.a"
    copy_changed "$output/libbibcitex_ffi.universal.a" "$output/libbibcitex_ffi.a"
    rm "$output/libbibcitex_ffi.universal.a"
fi
