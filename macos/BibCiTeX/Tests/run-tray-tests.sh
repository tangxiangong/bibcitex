#!/bin/bash
# Compile the independent tray model against a controllable service, without GUI.
set -euo pipefail
package="$(cd "$(dirname "$0")/.." && pwd)"
source_dir="$package/Sources/BibCiTeX"
work="$(mktemp -d "${TMPDIR:-/tmp}/bibcitex-tray.XXXXXX")"
trap 'rm -rf "$work"' EXIT
{
    cat "$source_dir/L10n.swift"
    cat "$source_dir/HelperModels.swift"
    sed '/^@MainActor/,$d' "$source_dir/WorkbenchModel.swift"
    cat "$source_dir/TrayWorkbenchModel.swift"
    cat "$package/Tests/TrayWorkbenchRegression.swift"
} > "$work/Regression.swift"
cp "$package/../../localization/"*.json "$work/"
swiftc -parse-as-library -target "$(uname -m)-apple-macosx14.0" \
    -module-cache-path "$work/modules" "$work/Regression.swift" -o "$work/regressions"
"$work/regressions"
