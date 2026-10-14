#!/bin/sh
# Uso (desde el repo): scripts/in-container.sh scripts/e2e/run.sh
set -eu
cd "$(dirname "$0")"
export XDG_RUNTIME_DIR=/tmp/e2e-runtime XDG_DATA_HOME=/tmp/e2e-data
rm -rf "$XDG_RUNTIME_DIR" "$XDG_DATA_HOME" out && mkdir -p -m 700 "$XDG_RUNTIME_DIR" && mkdir -p out
# El sway de Fedora tiene capacidades de archivo que un contenedor sin root no deja usar; la copia no las tiene.
cp /usr/bin/sway /tmp/sway
exec dbus-run-session -- sh -c '
    WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 /tmp/sway -c sway.conf >out/sway.log 2>&1 &
    for _ in $(seq 50); do [ -S "$XDG_RUNTIME_DIR/wayland-1" ] && break; sleep 0.1; done
    export WAYLAND_DISPLAY=wayland-1 SWAYSOCK="$(ls "$XDG_RUNTIME_DIR"/sway-ipc.*)" GDK_BACKEND=wayland
    python3 -I ui_test.py ../../src-tauri/target/release/patina
'
