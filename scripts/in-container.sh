#!/bin/sh
# Corre un comando dentro del contenedor de compilación, con el repo montado en /src.
# Las cachés de cargo, npm y xwin quedan en ~/.cache/patina-build para no repetir descargas.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/patina-build"
mkdir -p "$CACHE/registry" "$CACHE/git" "$CACHE/xwin" "$CACHE/npm"
podman image exists patina-build || podman build -t patina-build -f "$ROOT/build/Containerfile" "$ROOT/build"
exec podman run --rm --userns=keep-id -v "$ROOT:/src:Z" \
    -v "$CACHE/registry:/opt/cargo/registry:Z" -v "$CACHE/git:/opt/cargo/git:Z" \
    -v "$CACHE/xwin:/xwin-cache:Z" -e XWIN_CACHE_DIR=/xwin-cache \
    -v "$CACHE/npm:/npm-cache:Z" -e npm_config_cache=/npm-cache \
    patina-build "$@"
