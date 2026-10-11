#!/usr/bin/env bash
set -euo pipefail

case "${RUNNER_OS}/${RUNNER_ARCH}" in
    Linux/X64) platform=linux.x86_64 ;;
    Linux/ARM64) platform=linux.aarch64 ;;
    macOS/X64) platform=darwin.x86_64 ;;
    macOS/ARM64) platform=darwin.aarch64 ;;
    Windows/X64) platform=windows ;;
    *)
        printf 'Unsupported ShellCheck platform: %s/%s\n' "$RUNNER_OS" "$RUNNER_ARCH" >&2
        exit 1
        ;;
esac

install_dir=$(mktemp -d "${RUNNER_TEMP}/shellcheck.XXXXXX")
if [[ "$RUNNER_OS" == Windows ]]; then
    archive="shellcheck-v${SHELLCHECK_VERSION}.zip"
    curl --fail --silent --show-error --location --retry 3 \
        "https://github.com/koalaman/shellcheck/releases/download/v${SHELLCHECK_VERSION}/${archive}" \
        --output "$install_dir/$archive"
    unzip -q "$install_dir/$archive" -d "$install_dir"
else
    archive="shellcheck-v${SHELLCHECK_VERSION}.${platform}.tar.gz"
    curl --fail --silent --show-error --location --retry 3 \
        "https://github.com/koalaman/shellcheck/releases/download/v${SHELLCHECK_VERSION}/${archive}" \
        --output "$install_dir/$archive"
    tar -xzf "$install_dir/$archive" -C "$install_dir"
    mv "$install_dir/shellcheck-v${SHELLCHECK_VERSION}/shellcheck" "$install_dir/shellcheck"
fi
rm "$install_dir/$archive"

"$install_dir/shellcheck" --version

# GitHub expects a native Windows path, rather than Git Bash's /c/... form.
if [[ "$RUNNER_OS" == Windows ]]; then
    install_dir=$(cygpath -w "$install_dir")
fi
printf '%s\n' "$install_dir" >> "$GITHUB_PATH"
