#!/bin/sh
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/patina-build"
mkdir -p "$CACHE/registry" "$CACHE/git" "$CACHE/xwin" "$CACHE/npm" "$CACHE/home"
exec 9>"$CACHE/lock"
flock 9
podman image exists patina-build || podman build -t patina-build -f "$ROOT/build/Containerfile" "$ROOT/build"
podman run --rm --userns=keep-id -v "$ROOT:/src:Z" \
    -v "$CACHE/registry:/opt/cargo/registry:Z" -v "$CACHE/git:/opt/cargo/git:Z" \
    -v "$CACHE/xwin:/xwin-cache:Z" -e XWIN_CACHE_DIR=/xwin-cache \
    -v "$CACHE/npm:/npm-cache:Z" -e npm_config_cache=/npm-cache \
    -v "$CACHE/home:/home/builder:Z" -e HOME=/home/builder \
    patina-build "$@"
