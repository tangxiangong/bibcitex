#!/bin/bash
# Validate shortcut configuration without registering a hotkey or opening a GUI.
set -euo pipefail
package="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/bibcitex-shortcuts.XXXXXX")"
trap 'rm -rf "$work"' EXIT
swiftc -parse-as-library -target "$(uname -m)-apple-macosx13.0" \
    -module-cache-path "$work/modules" \
    "$package/Sources/BibCiTeX/HelperShortcut.swift" "$package/Tests/HelperShortcutRegression.swift" \
    -o "$work/regressions"
"$work/regressions"
