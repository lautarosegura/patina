#!/bin/sh
# Compila y prueba Patina dentro del contenedor de build (build/Containerfile).
#   scripts/build.sh test      tests de Rust (Linux) y del frontend
#   scripts/build.sh linux     instaladores .deb, .rpm y AppImage
#   scripts/build.sh windows   instalador NSIS (.exe) por compilación cruzada
#   scripts/build.sh all       todo lo anterior
# Los instaladores quedan en src-tauri/target/{release,x86_64-pc-windows-msvc/release}/bundle/.
set -eu
cd "$(dirname "$0")/.."
run() { ./scripts/in-container.sh sh -c "$1"; }
deps='[ -d node_modules ] || npm ci --no-audit --no-fund'

case "${1:-all}" in
test)
    run "$deps && node --test 'tests/ui/*.test.mjs' && cd src-tauri && cargo test --release"
    ;;
linux)
    # AppImage tools are AppImages themselves; without FUSE in the container they run extracted.
    run "$deps && APPIMAGE_EXTRACT_AND_RUN=1 npx tauri build --bundles deb,rpm,appimage"
    ;;
windows)
    run "$deps && npx tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis"
    ;;
all)
    "$0" test && "$0" linux && "$0" windows
    ;;
*)
    sed -n '2,7p' "$0" >&2
    exit 2
    ;;
esac
