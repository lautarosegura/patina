#!/bin/sh
# Instala Patina para el usuario actual. No usa sudo ni toca la configuración de Hyprland:
# al final indica la línea a agregar.
set -eu

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
SHARE_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/patina"
HOST_NAME="io.github.lautarosegura.patina"
EXTENSION_ID="fedemognlmdojgmeinfhhjliccpfncdo"  # deriva de la clave en extension/manifest.json
SRC="$(cd "$(dirname "$0")" && pwd)"

missing=""
python3 -c 'import gi; gi.require_version("Gtk", "4.0"); gi.require_version("Adw", "1")' 2>/dev/null \
    || missing="$missing python3-gobject gtk4 libadwaita"
for cmd in wl-copy wl-paste notify-send hyprctl; do
    command -v "$cmd" >/dev/null 2>&1 || missing="$missing $cmd"
done
if [ -n "$missing" ]; then
    echo "Faltan dependencias:$missing" >&2
    echo "En Fedora: sudo dnf install python3-gobject gtk4 libadwaita wl-clipboard libnotify" >&2
    exit 1
fi

install -Dm755 "$SRC/patina" "$BIN_DIR/patina"
mkdir -p "$SHARE_DIR/extension"
cp "$SRC"/extension/*.json "$SRC"/extension/*.js "$SHARE_DIR/extension/"
sed "s|@BIN@|$BIN_DIR/patina|g" "$SRC/hypr/patina.lua" > "$SHARE_DIR/hyprland.lua"
sed "s|@BIN@|$BIN_DIR/patina|g" "$SRC/hypr/patina.conf" > "$SHARE_DIR/hyprland.conf"

# Host nativo de la extensión, para cada navegador Chromium que esté instalado.
hosts=""
for dir in "$HOME/.config/BraveSoftware/Brave-Browser" "$HOME/.config/google-chrome" "$HOME/.config/chromium"; do
    [ -d "$dir" ] || continue
    mkdir -p "$dir/NativeMessagingHosts"
    cat > "$dir/NativeMessagingHosts/$HOST_NAME.json" <<JSON
{
  "name": "$HOST_NAME",
  "description": "Patina: recibe la página y el remitente del texto resaltado",
  "path": "$BIN_DIR/patina",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://$EXTENSION_ID/"]
}
JSON
    hosts="$hosts\n  $dir"
done

# Reinicia el servicio si ya estaba corriendo, para usar la versión nueva.
pkill -f "^python3 $BIN_DIR/patina" 2>/dev/null || true
if [ -n "${HYPRLAND_INSTANCE_SIGNATURE:-}" ]; then
    hyprctl dispatch "hl.dsp.exec_cmd(\"$BIN_DIR/patina --service\")" >/dev/null 2>&1 \
        || hyprctl dispatch exec "$BIN_DIR/patina --service" >/dev/null 2>&1 || true
fi

cat <<MSG
Patina instalado en $BIN_DIR/patina

1. Hyprland: agregá al final de tu configuración
   hyprland.lua:  dofile(os.getenv("HOME") .. "/.local/share/patina/hyprland.lua")
   hyprland.conf: source = $SHARE_DIR/hyprland.conf
   y recargá con: hyprctl reload

2. Extensión del navegador (opcional, para guardar la página, el favicon y el remitente de WhatsApp):
   abrí brave://extensions (o chrome://extensions), activá el modo de desarrollador,
   "Cargar extensión sin empaquetar" y elegí: $SHARE_DIR/extension
MSG
[ -n "$hosts" ] && printf "   Host nativo instalado en:$hosts\n" || echo "   (No encontré Brave, Chrome ni Chromium: el host nativo no se instaló)"
