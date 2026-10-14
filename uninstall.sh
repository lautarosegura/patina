#!/bin/sh
# Uninstalls Patina. Your captures and notes stay in ~/.local/share/patina unless you pass --purge.
set -eu

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/patina"
HOST_NAME="io.github.lautarosegura.patina"

pkill -f "^python3 $BIN_DIR/patina" 2>/dev/null || true
rm -f "$BIN_DIR/patina"
for dir in "$HOME/.config/BraveSoftware/Brave-Browser" "$HOME/.config/google-chrome" "$HOME/.config/chromium"; do
    rm -f "$dir/NativeMessagingHosts/$HOST_NAME.json"
done
if [ "${1:-}" = "--purge" ]; then
    rm -rf "$DATA_DIR"
    echo "Patina uninstalled, including your data."
else
    echo "Patina uninstalled. Your data is still in $DATA_DIR (use --purge to delete it)."
fi
echo "Also remove the Patina line from your Hyprland config and the browser extension."
