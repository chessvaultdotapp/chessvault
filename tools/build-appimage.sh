#!/usr/bin/env bash
# Build a native Linux x86_64 AppImage from any working directory.
set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root"

if [[ $(uname -s) != Linux || $(uname -m) != x86_64 ]]; then
    echo 'AppImage packaging currently requires Linux x86_64.' >&2
    exit 1
fi
for tool in cargo curl sha256sum desktop-file-validate; do
    command -v "$tool" >/dev/null || { echo "Missing required tool: $tool" >&2; exit 1; }
done

# This linuxdeploy release includes its AppImage output plugin.
release=1-alpha-20251107-1
checksum=c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d
cache="$root/target/appimage-tools"
mkdir -p "$cache" "$root/dist"
linuxdeploy="$cache/linuxdeploy-x86_64.AppImage"
if [[ ! -f "$linuxdeploy" ]]; then
    curl --fail --location --retry 3 \
        "https://github.com/linuxdeploy/linuxdeploy/releases/download/$release/linuxdeploy-x86_64.AppImage" \
        --output "$linuxdeploy.part"
    mv -- "$linuxdeploy.part" "$linuxdeploy"
fi
printf '%s  %s\n' "$checksum" "$linuxdeploy" | sha256sum --check --status
chmod +x "$linuxdeploy"

# Avoid the output plugin's default download of a mutable continuous runtime.
runtime="$cache/runtime-x86_64"
if [[ ! -f "$runtime" ]]; then
    curl --fail --location --retry 3 \
        'https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64' \
        --output "$runtime.part"
    mv -- "$runtime.part" "$runtime"
fi
printf '%s  %s\n' '2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d' "$runtime" \
    | sha256sum --check --status
export LDAI_RUNTIME_FILE="$runtime"

desktop="$root/apps/chessvault/data/chessvault.desktop"
desktop-file-validate "$desktop"
# Explicit target and target-dir avoid accidentally packaging a cross-compiled binary.
triple=x86_64-unknown-linux-gnu
cargo build -p chessvault --release --locked --target "$triple" --target-dir "$root/target"

# Use a fresh staging directory so removed dependencies cannot leak into later builds.
stage=$(mktemp -d "$root/target/appimage.XXXXXX")
trap 'rm -rf -- "$stage"' EXIT
export APPIMAGE_EXTRACT_AND_RUN=1
export ARCH=x86_64
export OUTPUT="$root/dist/ChessVault-x86_64.AppImage"
cd "$stage"
"$linuxdeploy" \
    --appdir "$stage/AppDir" \
    --executable "$root/target/$triple/release/chessvault" \
    --desktop-file "$desktop" \
    --icon-file "$root/apps/chessvault/data/chessvault.svg" \
    --output appimage
printf '\nBuilt %s\n' "$OUTPUT"
