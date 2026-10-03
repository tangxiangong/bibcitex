#!/bin/bash
# User-invoked only: this compiles the binding generator, then generates bindings.
set -euo pipefail
repo="$(cd "$(dirname "$0")/.." && pwd)"
library="${1:-$repo/target/release/libbibcitex_ffi.dylib}"
if [[ ! -f "$library" ]]; then
    echo "Missing Rust library: $library. Build bibcitex-ffi first." >&2
    exit 1
fi
cd "$repo"
cargo run --locked --release -p bibcitex-bindgen -- generate \
    "$library" --language swift --no-format \
    --out-dir "$repo/bindings/generated/swift"
