#!/usr/bin/env bash
set -euo pipefail

case "${RUNNER_OS}/${RUNNER_ARCH}" in
  Linux/X64) target=x86_64-unknown-linux-musl ;;
  Linux/ARM64) target=aarch64-unknown-linux-musl ;;
  macOS/X64) target=x86_64-apple-darwin ;;
  macOS/ARM64) target=aarch64-apple-darwin ;;
  Windows/X64) target=x86_64-pc-windows-msvc ;;
  Windows/ARM64) target=aarch64-pc-windows-msvc ;;
  *)
    printf 'Unsupported Tombi platform: %s/%s\n' "$RUNNER_OS" "$RUNNER_ARCH" >&2
    exit 1
    ;;
esac

extension=tar.gz
if [[ "$RUNNER_OS" == Windows ]]; then
  extension=zip
fi

install_dir=$(mktemp -d "${RUNNER_TEMP}/tombi.XXXXXX")
release_dir="tombi-cli-${TOMBI_VERSION}-${target}"
archive="${release_dir}.${extension}"
curl --fail --silent --show-error --location --retry 3 \
  "https://github.com/tombi-toml/tombi/releases/download/v${TOMBI_VERSION}/${archive}" \
  --output "$install_dir/$archive"
if [[ "$RUNNER_OS" == Windows ]]; then
  unzip -q "$install_dir/$archive" -d "$install_dir"
else
  tar -xzf "$install_dir/$archive" -C "$install_dir"
fi
rm "$install_dir/$archive"

# Tarballs contain a release directory; Windows ZIPs contain the binary directly.
if [[ "$RUNNER_OS" != Windows ]]; then
  install_dir="$install_dir/$release_dir"
fi
"$install_dir/tombi" --version

# GitHub expects a native Windows path, rather than Git Bash's /c/... form.
if [[ "$RUNNER_OS" == Windows ]]; then
  install_dir=$(cygpath -w "$install_dir")
fi
printf '%s\n' "$install_dir" >> "$GITHUB_PATH"
