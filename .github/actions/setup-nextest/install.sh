#!/usr/bin/env bash
set -euo pipefail

case "${RUNNER_OS}/${RUNNER_ARCH}" in
  Linux/X64) platform=linux-musl ;;
  Linux/ARM64) platform=linux-arm-musl ;;
  macOS/X64|macOS/ARM64) platform=mac ;;
  Windows/X64) platform=windows-tar ;;
  Windows/ARM64) platform=windows-arm-tar ;;
  *)
    printf 'Unsupported nextest platform: %s/%s\n' "$RUNNER_OS" "$RUNNER_ARCH" >&2
    exit 1
    ;;
esac

install_dir=$(mktemp -d "${RUNNER_TEMP}/nextest.XXXXXX")
curl --fail --silent --show-error --location --retry 3 \
  "https://get.nexte.st/${NEXTEST_VERSION}/${platform}" \
  --output "$install_dir/nextest.tar.gz"
tar -xzf "$install_dir/nextest.tar.gz" -C "$install_dir"
rm "$install_dir/nextest.tar.gz"

"$install_dir/cargo-nextest" nextest --version

# GitHub expects a native Windows path, rather than Git Bash's /c/... form.
if [[ "$RUNNER_OS" == Windows ]]; then
  install_dir=$(cygpath -w "$install_dir")
fi
printf '%s\n' "$install_dir" >> "$GITHUB_PATH"
