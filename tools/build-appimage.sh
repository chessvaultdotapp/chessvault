#!/usr/bin/env bash
# Build a native Linux x86_64 AppImage from any working directory.
set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root"

if [[ $(uname -s) != Linux || $(uname -m) != x86_64 ]]; then
    echo 'AppImage packaging currently requires Linux x86_64.' >&2
    exit 1
fi
for tool in cargo curl sha256sum desktop-file-validate readelf; do
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
# Match Icon=chessvault without maintaining a second copy of the logo.
cp -- "$root/apps/chessvault/assets/logo.svg" "$stage/chessvault.svg"
export APPIMAGE_EXTRACT_AND_RUN=1
export ARCH=x86_64
# Build privately; a failed packaging run must not replace the previous artifact.
export OUTPUT="$stage/ChessVault-x86_64.AppImage"
mkdir -p "$stage/AppDir/usr/share/licenses/chessvault"
cp -- "$root/LICENSE" "$stage/AppDir/usr/share/licenses/chessvault/LICENSE"
cd "$stage"
"$linuxdeploy" \
    --appdir "$stage/AppDir" \
    --executable "$root/target/$triple/release/chessvault" \
    --desktop-file "$desktop" \
    --icon-file "$stage/chessvault.svg" \
    --output appimage
# Exercise the generated runtime without FUSE and check the packaged entry point.
"$OUTPUT" --appimage-extract >/dev/null
test -x squashfs-root/AppRun
test -x squashfs-root/usr/bin/chessvault
desktop-file-validate squashfs-root/chessvault.desktop
test -s squashfs-root/usr/share/licenses/chessvault/LICENSE
# Record the actual glibc requirement for release review (including bundled libraries).
find squashfs-root -type f -exec readelf --version-info {} \; 2>/dev/null \
    | grep -oE 'GLIBC_[0-9]+\.[0-9]+' | sort -Vu > "$stage/glibc-requirements.txt"
cp -- "$stage/glibc-requirements.txt" "$root/dist/ChessVault-x86_64.glibc.txt"
chmod 755 "$OUTPUT"
mv -- "$OUTPUT" "$root/dist/ChessVault-x86_64.AppImage"
cd "$root/dist"
sha256sum ChessVault-x86_64.AppImage > ChessVault-x86_64.AppImage.sha256
printf '\nBuilt %s\n' "$root/dist/ChessVault-x86_64.AppImage"
