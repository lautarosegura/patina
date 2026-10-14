#!/bin/sh
# Desinstala Patina. Tus capturas y notas quedan en ~/.local/share/patina salvo que pases --purge.
set -eu

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
SHARE_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/patina"
HOST_NAME="io.github.lautarosegura.patina"

pkill -f "^python3 $BIN_DIR/patina" 2>/dev/null || true
rm -f "$BIN_DIR/patina"
for dir in "$HOME/.config/BraveSoftware/Brave-Browser" "$HOME/.config/google-chrome" "$HOME/.config/chromium"; do
    rm -f "$dir/NativeMessagingHosts/$HOST_NAME.json"
done
rm -rf "$SHARE_DIR/extension" "$SHARE_DIR/hyprland.lua" "$SHARE_DIR/hyprland.conf"
if [ "${1:-}" = "--purge" ]; then
    rm -rf "$SHARE_DIR"
    echo "Patina desinstalado, incluidos tus datos."
else
    echo "Patina desinstalado. Tus datos siguen en $SHARE_DIR (usá --purge para borrarlos)."
fi
echo "Quitá también la línea de Patina de tu configuración de Hyprland y la extensión del navegador."
