#!/usr/bin/env bash
set -euo pipefail

case "${RUNNER_OS}/${RUNNER_ARCH}" in
    Linux/X64) asset=linux_amd64 ;;
    Linux/ARM64) asset=linux_arm64 ;;
    macOS/X64) asset=darwin_amd64 ;;
    macOS/ARM64) asset=darwin_arm64 ;;
    Windows/X64) asset=windows_amd64.exe ;;
    *)
        printf 'Unsupported shfmt platform: %s/%s\n' "$RUNNER_OS" "$RUNNER_ARCH" >&2
        exit 1
        ;;
esac

install_dir=$(mktemp -d "${RUNNER_TEMP}/shfmt.XXXXXX")
binary="shfmt_v${SHFMT_VERSION}_${asset}"
curl --fail --silent --show-error --location --retry 3 \
    "https://github.com/mvdan/sh/releases/download/v${SHFMT_VERSION}/${binary}" \
    --output "$install_dir/$binary"
if [[ "$RUNNER_OS" == Windows ]]; then
    mv "$install_dir/$binary" "$install_dir/shfmt.exe"
else
    mv "$install_dir/$binary" "$install_dir/shfmt"
    chmod +x "$install_dir/shfmt"
fi

"$install_dir/shfmt" --version

# GitHub expects a native Windows path, rather than Git Bash's /c/... form.
if [[ "$RUNNER_OS" == Windows ]]; then
    install_dir=$(cygpath -w "$install_dir")
fi
printf '%s\n' "$install_dir" >> "$GITHUB_PATH"
