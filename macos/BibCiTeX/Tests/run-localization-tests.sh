#!/bin/bash
# Exercise production localization without launching a GUI or changing preferences.
set -euo pipefail
package="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/bibcitex-localization.XXXXXX")"
trap 'rm -rf "$work"' EXIT
cp "$package/../../localization/"*.json "$work/"
swiftc -parse-as-library -target "$(uname -m)-apple-macosx13.0" \
    -module-cache-path "$work/modules" \
    "$package/Sources/BibCiTeX/L10n.swift" "$package/Sources/BibCiTeX/MainMenuLocalizer.swift" \
    "$package/Tests/LocalizationRegression.swift" \
    -o "$work/localization-tests"
"$work/localization-tests"
