#!/bin/bash
# Exercise the production workbench model with a controllable actor; no Rust/GUI.
set -euo pipefail
package="$(cd "$(dirname "$0")/.." && pwd)"
source_dir="$package/Sources/BibCiTeX"
work="$(mktemp -d "${TMPDIR:-/tmp}/bibcitex-workbench.XXXXXX")"
trap 'rm -rf "$work"' EXIT
{
    cat "$source_dir/L10n.swift"
    cat "$source_dir/HelperModels.swift"
    sed '/^actor RustWorkbenchService:/,$d' "$source_dir/WorkbenchModel.swift"
    cat "$package/Tests/WorkbenchModelRegression.swift"
} > "$work/Regression.swift"
cp "$package/../../localization/"*.json "$work/"
swiftc -parse-as-library -target "$(uname -m)-apple-macosx13.0" \
    -module-cache-path "$work/modules" "$work/Regression.swift" -o "$work/regressions"
"$work/regressions"
