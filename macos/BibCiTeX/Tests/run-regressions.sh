#!/bin/bash
# Compile and run the production model with an actor test double; no Rust or GUI.
set -euo pipefail
package="$(cd "$(dirname "$0")/.." && pwd)"
source_dir="$package/Sources/BibCiTeX"
work="$(mktemp -d "${TMPDIR:-/tmp}/bibcitex-regression.XXXXXX")"
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
{
    cat "$source_dir/L10n.swift"
    cat "$source_dir/HelperModels.swift"
    # Only the theme declarations are needed, not the generated Rust service.
    sed '/^actor HelperService:/,$d' "$source_dir/HelperTheme.swift"
    cat "$source_dir/HelperViewModel.swift"
    cat "$package/Tests/HelperViewModelRegression.swift"
} > "$work/Regression.swift"
cp "$package/../../localization/"*.json "$work/"
swiftc -parse-as-library -target "$(uname -m)-apple-macosx13.0" \
    -module-cache-path "$work/modules" "$work/Regression.swift" -o "$work/regressions"
"$work/regressions" &
test_pid=$!
(
    sleep 20
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
    echo "Helper regression tests timed out after 20 seconds" >&2
    exit 124
fi
exit "$status"
