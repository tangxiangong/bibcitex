#!/bin/bash
# Link/run generated Swift/Rust ABI tests without starting the application.
set -euo pipefail
package="$(cd "$(dirname "$0")/.." && pwd)"
repo="$(cd "$package/../.." && pwd)"
architecture="$(uname -m)"
products="$repo/target/xcode/$architecture/Build/Products/Debug"
rust_library=""
usage() { echo "Usage: $0 [--products PATH] [--rust-library PATH]"; }
while [[ $# -gt 0 ]]; do
    case "$1" in
        --products|--rust-library)
            [[ $# -ge 2 ]] || { usage >&2; exit 2; }
            if [[ "$1" == --products ]]; then products="$2"; else rust_library="$2"; fi
            shift 2
            ;;
        --products=*) products="${1#*=}"; shift;;
        --rust-library=*) rust_library="${1#*=}"; shift;;
        -h|--help) usage; exit 0;;
        *) usage >&2; exit 2;;
    esac
done
bindings="$(dirname "$products")/BibCiTeXBindings/$(basename "$products")"
rust_library="${rust_library:-$bindings/libbibcitex_ffi.a}"
for required in "$products/libBibCiTeXCore.a" "$rust_library" "$bindings/include/module.modulemap"; do
    [[ -f "$required" ]] || { echo "Build the Swift app and Rust library first: missing $required" >&2; exit 1; }
done
work="$(mktemp -d "${TMPDIR:-/tmp}/bibcitex-interop-tests.XXXXXX")"
watchdog=""
test_pid=""
cleanup() {
    if [[ -n "$watchdog" ]]; then kill "$watchdog" 2>/dev/null || true; fi
    if [[ -n "$test_pid" ]]; then kill "$test_pid" 2>/dev/null || true; fi
    rm -rf "$work"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
swiftc -parse-as-library -target "$architecture-apple-macosx13.0" \
    -module-cache-path "$work/modules" \
    -I "$products" -I "$bindings/include" \
    "$package/Tests/RustInteropTests.swift" "$products/libBibCiTeXCore.a" "$rust_library" \
    -framework AppKit -framework Carbon -framework ApplicationServices \
    -framework CoreGraphics -framework Foundation -liconv -o "$work/interop-tests"
"$work/interop-tests" &
test_pid=$!
(
    sleep 30
    if kill -0 "$test_pid" 2>/dev/null; then
        touch "$work/timed-out"
        kill -KILL "$test_pid" 2>/dev/null || true
    fi
) >/dev/null 2>&1 &
watchdog=$!
status=0
wait "$test_pid" || status=$?
test_pid=""
kill "$watchdog" 2>/dev/null || true
wait "$watchdog" 2>/dev/null || true
watchdog=""
if [[ -f "$work/timed-out" ]]; then
    echo "Swift/Rust interop tests timed out after 30 seconds" >&2
    exit 124
fi
exit "$status"
