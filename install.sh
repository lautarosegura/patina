#!/bin/sh
# Installs Patina for the current user from this clone: the command is a symlink to the file in
# the repo, and the extension and Hyprland config are used from here. If you move or delete the
# clone, run install.sh again. Doesn't use sudo and doesn't edit your Hyprland config.
set -eu

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
HOST_NAME="io.github.lautarosegura.patina"
EXTENSION_ID="fedemognlmdojgmeinfhhjliccpfncdo"  # derived from the key in extension/manifest.json
SRC="$(cd "$(dirname "$0")" && pwd)"

missing=""
python3 -c 'import gi; gi.require_version("Gtk", "4.0"); gi.require_version("Adw", "1")' 2>/dev/null \
    || missing="$missing python3-gobject gtk4 libadwaita"
for cmd in wl-copy wl-paste notify-send hyprctl; do
    command -v "$cmd" >/dev/null 2>&1 || missing="$missing $cmd"
done
if [ -n "$missing" ]; then
    echo "Missing dependencies:$missing" >&2
    echo "Fedora: sudo dnf install python3-gobject gtk4 libadwaita wl-clipboard libnotify" >&2
    echo "Arch:   sudo pacman -S python-gobject gtk4 libadwaita wl-clipboard libnotify" >&2
    exit 1
fi

mkdir -p "$BIN_DIR"
ln -sfn "$SRC/patina" "$BIN_DIR/patina"

# Native messaging host for the extension, for every installed Chromium-based browser.
hosts=""
for dir in "$HOME/.config/BraveSoftware/Brave-Browser" "$HOME/.config/google-chrome" "$HOME/.config/chromium"; do
    [ -d "$dir" ] || continue
    mkdir -p "$dir/NativeMessagingHosts"
    cat > "$dir/NativeMessagingHosts/$HOST_NAME.json" <<JSON
{
  "name": "$HOST_NAME",
  "description": "Patina: receives the page and sender of highlighted text",
  "path": "$BIN_DIR/patina",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://$EXTENSION_ID/"]
}
JSON
    hosts="$hosts\n  $dir"
done

# Restart the service if it was running, so it picks up this version.
pkill -f "^python3 $BIN_DIR/patina" 2>/dev/null || true
if [ -n "${HYPRLAND_INSTANCE_SIGNATURE:-}" ]; then
    hyprctl dispatch "hl.dsp.exec_cmd(\"$BIN_DIR/patina --service\")" >/dev/null 2>&1 \
        || hyprctl dispatch exec "$BIN_DIR/patina --service" >/dev/null 2>&1 || true
fi

# Which Hyprland file to include, by version and config format.
# "hyprctl version" changed format across releases; the number is always there (e.g. "Tag: v0.45.0").
version="$(hyprctl version 2>/dev/null | grep -o '0\.[0-9][0-9]*\.[0-9][0-9]*' | head -n1)"
minor="$(echo "${version:-0.0}" | cut -d. -f2)"
conf_dir="${XDG_CONFIG_HOME:-$HOME/.config}/hypr"
if [ -f "$conf_dir/hyprland.lua" ] && [ "$minor" -ge 55 ]; then
    extra=""
    [ "$BIN_DIR" = "$HOME/.local/bin" ] || extra="PATINA_BIN = \"$BIN_DIR/patina\"; "
    include="${extra}dofile(\"$SRC/hypr/patina.lua\")"
    target="$conf_dir/hyprland.lua"
else
    if [ "$minor" -ge 53 ]; then file=patina.conf
    elif [ "$minor" -ge 48 ]; then file=patina-0.48.conf
    else file=patina-0.47.conf
    fi
    include="source = $SRC/hypr/$file"
    target="$conf_dir/hyprland.conf"
    [ "$BIN_DIR" = "$HOME/.local/bin" ] || include="$include   (replace ~/.local/bin/patina with $BIN_DIR/patina in that file)"
fi

cat <<MSG
Patina installed: $BIN_DIR/patina -> $SRC/patina

1. Hyprland ${version:-(unknown version)}: add this at the end of $target
   $include
   then reload with: hyprctl reload

2. Browser extension (optional: saves the page, favicon and WhatsApp sender):
   open brave://extensions (or chrome://extensions), turn on Developer mode,
   click "Load unpacked" and pick: $SRC/extension
MSG
[ -n "$hosts" ] && printf "   Native host installed for:$hosts\n" || echo "   (No Brave, Chrome or Chromium found: native host not installed)"
