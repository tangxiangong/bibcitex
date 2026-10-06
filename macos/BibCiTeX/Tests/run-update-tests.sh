#!/bin/bash
# Test update version policy without starting Sparkle or a GUI.
set -euo pipefail
package="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/bibcitex-updates.XXXXXX")"
trap 'rm -rf "$work"' EXIT
sed '/^@MainActor/,$d; /^import Combine$/d; /^import Sparkle$/d' "$package/Sources/BibCiTeX/Updater.swift" > "$work/Policy.swift"
swiftc -parse-as-library -target "$(uname -m)-apple-macosx14.0" \
    -module-cache-path "$work/modules" "$work/Policy.swift" "$package/Tests/UpdatePolicyRegression.swift" -o "$work/regressions"
"$work/regressions"
