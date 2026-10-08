#!/bin/sh
set -eu
bundle_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
install_dir="${XDG_DATA_HOME:-$HOME/.local/share}/bibcitex"
bin_dir="$HOME/.local/bin"
applications_dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
icons_dir="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/128x128/apps"
mkdir -p "$install_dir" "$bin_dir" "$applications_dir" "$icons_dir"
install -m 755 "$bundle_dir/bin/bibcitex" "$install_dir/bibcitex.new"
mv -f "$install_dir/bibcitex.new" "$install_dir/bibcitex"
ln -sfn "$install_dir/bibcitex" "$bin_dir/bibcitex"
# Desktop Exec uses an absolute quoted path and does not depend on shell PATH.
escaped_executable=$(printf '%s' "$install_dir/bibcitex" | sed 's/[\\"`$]/\\&/g; s/%/%%/g')
while IFS= read -r line; do
    case "$line" in
        Exec=bibcitex*) printf 'Exec="%s"%s\n' "$escaped_executable" "${line#Exec=bibcitex}" ;;
        *) printf '%s\n' "$line" ;;
    esac
done < "$bundle_dir/share/applications/io.github.tangxiangong.bibcitex.desktop" > "$applications_dir/io.github.tangxiangong.bibcitex.desktop"
install -m 644 "$bundle_dir/share/icons/hicolor/128x128/apps/io.github.tangxiangong.bibcitex.png" "$icons_dir/io.github.tangxiangong.bibcitex.png"
extension_dir="${XDG_DATA_HOME:-$HOME/.local/share}/gnome-shell/extensions/bibcitex-tray@tangxiangong.github.io"
mkdir -p "$extension_dir"
for resource in extension.js metadata.json; do
    install -m 644 "$bundle_dir/share/gnome-shell/extensions/bibcitex-tray@tangxiangong.github.io/$resource" "$extension_dir/$resource"
done
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$applications_dir" || true
printf '%s\n' "Installed BibCiTeX to $install_dir"
