#!/bin/sh
set -eu

prefix="${HOME:?HOME must be set}/.local"
action=install
dry_run=false

usage() {
    # Literal "$HOME" and "$XDG_DATA_HOME" names are shown in the help text.
    # shellcheck disable=SC2016
    printf '%s\n' \
        'Usage: install-linux.sh [--prefix PATH] [--uninstall] [--dry-run]' \
        '       install-linux.sh --help' \
        '' \
        'Options:' \
        '  --prefix PATH  Install only the executable into PATH/bin (default: $HOME/.local).' \
        '                 Does not change desktop entry or icon locations.' \
        '  --dry-run      Preview installation without changing files.' \
        '  --uninstall    Not implemented yet.' \
        '  --help         Show this help.' \
        '' \
        'Desktop entries and icons use $XDG_DATA_HOME (default: $HOME/.local/share).' \
        'Relative XDG_DATA_HOME values are ignored.' \
        'If a custom XDG_DATA_HOME differs from your desktop session setting,' \
        'the desktop environment may not discover the launcher or icons.'
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix)
            if [ "$#" -lt 2 ] || [ -z "$2" ]; then
                printf '%s\n' 'Error: --prefix requires a path.' >&2
                exit 1
            fi
            prefix="$2"
            shift 2
            ;;
        --dry-run)
            dry_run=true
            shift
            ;;
        --uninstall)
            action=uninstall
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            printf 'Error: unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

case "$prefix" in
    /*) ;;
    *)
        printf '%s\n' 'Error: --prefix must be an absolute path.' >&2
        exit 1
        ;;
esac

# Reject common system-wide prefixes, including repeated or trailing slashes
# and dot components. realpath -m does not require the destination to exist.
if ! resolved_prefix=$(realpath -m -- "$prefix"); then
    printf 'Error: cannot resolve prefix: %s\n' "$prefix" >&2
    exit 1
fi

case "$resolved_prefix" in
    / | /usr | /usr/* | /opt | /opt/* | /bin | /bin/* | /sbin | /sbin/* | /lib | /lib/* | /lib64 | /lib64/*)
        # The suggested prefix is shown literally in the error message.
        # shellcheck disable=SC2016
        printf '%s\n' \
            'Error: this script does not support system-wide installation.' \
            'The ChessVault developers do not recommend system-wide installation.' \
            'Choose a per-user prefix, such as --prefix "$HOME/.local".' >&2
        exit 1
        ;;
esac

# Confirm non-default prefixes before a real installation. Dry runs do not
# modify anything and remain usable without interactive input.
default_prefix=$(realpath -m -- "$HOME/.local")
if [ "$action" = install ] && [ "$dry_run" = false ] &&
    [ "$resolved_prefix" != "$default_prefix" ]; then
    if [ ! -t 0 ]; then
        printf '%s\n' \
            'Error: a custom prefix requires interactive confirmation.' \
            'Run from a terminal, or use --dry-run to preview the installation.' >&2
        exit 1
    fi
    printf 'Install into custom prefix "%s"? [y/N] ' "$resolved_prefix" >&2
    answer=
    if ! IFS= read -r answer; then
        printf '\nInstallation cancelled.\n' >&2
        exit 1
    fi
    case "$answer" in
        y | Y | yes | YES | Yes)
            printf '%s\n' \
                'Warning: a custom prefix may not be on PATH, so terminal commands and tools' \
                'that search PATH may not find ChessVault.' \
                'The desktop launcher uses the absolute executable path and should still work.' >&2
            ;;
        *)
            printf '%s\n' 'Installation cancelled.' >&2
            exit 1
            ;;
    esac
fi

# Resolve paths relative to this script, not the caller's directory.
package_dir=$(
    CDPATH='' cd -- "$(dirname -- "$0")" && pwd -P
)

binary_source="$package_dir/bin/chessvault"
desktop_source="$package_dir/share/applications/chessvault.desktop"
png_source="$package_dir/share/icons/hicolor/256x256/apps/chessvault.png"
svg_source="$package_dir/share/icons/hicolor/scalable/apps/chessvault.svg"
license_source="$package_dir/share/licenses/chessvault/LICENSE"

# Uninstall should not require the original payload.
if [ "$action" = install ]; then
    for source in \
        "$binary_source" \
        "$desktop_source" \
        "$png_source" \
        "$svg_source" \
        "$license_source"; do
        if [ ! -f "$source" ] || [ ! -r "$source" ]; then
            printf 'Error: missing or unreadable package file: %s\n' \
                "$source" >&2
            printf '%s\n' \
                'Run the installer from a fully extracted ChessVault package.' >&2
            exit 1
        fi
    done

    printf 'Package files found in: %s\n' "$package_dir"
fi

# Route future filesystem-changing commands through this helper.
# Arguments are printed separately to keep paths containing spaces readable.
run() {
    if [ "$dry_run" = true ]; then
        printf '[dry-run]'
        for arg in "$@"; do
            printf ' <%s>' "$arg"
        done
        printf '\n'
    else
        "$@"
    fi
}

if [ "$action" = uninstall ]; then
    printf '%s\n' 'Error: uninstall is not implemented yet.' >&2
    exit 1
fi

# XDG requires absolute paths; ignore relative values as the specification says.
case "${XDG_DATA_HOME:-}" in
    /*) data_home=$XDG_DATA_HOME ;;
    *) data_home="$HOME/.local/share" ;;
esac
case "$data_home" in
    /*) ;;
    *)
        printf '%s\n' 'Error: HOME must be absolute when XDG_DATA_HOME is not absolute.' >&2
        exit 1
        ;;
esac

if [ "$data_home" != "$HOME/.local/share" ]; then
    printf '%s\n' \
        'Warning: using a custom XDG_DATA_HOME for the desktop entry and icons.' \
        'If your desktop session uses a different XDG_DATA_HOME, the desktop' \
        'environment may not discover the launcher or icons.' >&2
fi

binary_destination="$resolved_prefix/bin/chessvault"
desktop_destination="$data_home/applications/chessvault.desktop"
png_destination="$data_home/icons/hicolor/256x256/apps/chessvault.png"
svg_destination="$data_home/icons/hicolor/scalable/apps/chessvault.svg"
license_destination="$data_home/licenses/chessvault/LICENSE"
manifest_destination="$data_home/chessvault/install-manifest-v1"

# Keep the tab-separated manifest unambiguous, including for non-Unicode paths.
for root in "$resolved_prefix" "$data_home"; do
    case "$root" in
        *"$(printf '\t')"* | *'
'* | *"$(printf '\r')"*)
            printf '%s\n' 'Error: installation roots must not contain tabs or line breaks.' >&2
            exit 1
            ;;
    esac
done
if ! command -v sha256sum > /dev/null 2>&1; then
    printf '%s\n' 'Error: sha256sum is required to record installed files.' >&2
    exit 1
fi

# Check every destination before modifying anything. Until upgrade support is
# implemented, even existing recorded ChessVault installations are refused.
for destination in \
    "$binary_destination" \
    "$desktop_destination" \
    "$png_destination" \
    "$svg_destination" \
    "$license_destination" \
    "$manifest_destination"; do
    if [ -e "$destination" ] || [ -L "$destination" ]; then
        printf 'Error: refusing to overwrite existing path: %s\n' "$destination" >&2
        exit 1
    fi
    parent=$(dirname -- "$destination")
    while [ ! -e "$parent" ] && [ ! -L "$parent" ]; do
        parent=$(dirname -- "$parent")
    done
    if [ ! -d "$parent" ] || [ ! -w "$parent" ] || [ ! -x "$parent" ]; then
        printf 'Error: destination parent is not a writable directory: %s\n' "$parent" >&2
        printf '%s\n' 'Choose a writable per-user prefix and XDG_DATA_HOME.' >&2
        exit 1
    fi
done

# Quote Exec for both desktop-entry string parsing and Exec argument parsing.
# Passing through the environment avoids awk -v backslash interpretation.
render_desktop() {
    CHESSVAULT_EXEC="$binary_destination" awk '
        BEGIN {
            path = ENVIRON["CHESSVAULT_EXEC"]
            escaped = ""
            for (i = 1; i <= length(path); i++) {
                c = substr(path, i, 1)
                if (c == "\\") escaped = escaped "\\\\\\\\"
                else if (c == "\"" || c == "$" || c == "`") escaped = escaped "\\\\" c
                else if (c == "%") escaped = escaped "%%"
                else if (c == "\t") escaped = escaped "\\t"
                else escaped = escaped c
            }
        }
        /^Exec=/ { print "Exec=\"" escaped "\""; next }
        { print }
    ' "$desktop_source"
}

if [ "$dry_run" = true ]; then
    printf 'Would install executable: %s\n' "$binary_destination"
    printf 'Would install desktop entry: %s\n' "$desktop_destination"
    printf 'Would install PNG icon: %s\n' "$png_destination"
    printf 'Would install SVG icon: %s\n' "$svg_destination"
    printf 'Would install license: %s\n' "$license_destination"
    printf 'Would record installation: %s\n' "$manifest_destination"
    printf '%s\n' 'Generated desktop entry:'
    render_desktop
    exit 0
fi

temporary_desktop=$(mktemp)
temporary_manifest=
trap 'rm -f -- "$temporary_desktop"; if [ -n "$temporary_manifest" ]; then rm -f -- "$temporary_manifest"; fi' EXIT
trap 'exit 1' HUP INT TERM
render_desktop > "$temporary_desktop"

install_file() {
    mode=$1
    source=$2
    destination=$3
    run mkdir -p -- "$(dirname -- "$destination")"
    # noclobber also protects against files created since the preflight check.
    if ! (
        set -C
        umask 077
        : > "$destination"
    ); then
        printf 'Error: could not reserve destination: %s\n' "$destination" >&2
        exit 1
    fi
    if ! run install -m "$mode" -- "$source" "$destination"; then
        printf 'Error: failed to install: %s\n' "$destination" >&2
        printf '%s\n' 'Installation may be partial; inspect the reported destinations before retrying.' >&2
        exit 1
    fi
}

install_file 0755 "$binary_source" "$binary_destination"
install_file 0644 "$temporary_desktop" "$desktop_destination"
install_file 0644 "$png_source" "$png_destination"
install_file 0644 "$svg_source" "$svg_destination"
install_file 0644 "$license_source" "$license_destination"

# Record only the five fixed file identifiers, never executable shell content.
# Hash stdin so sha256sum does not escape unusual filenames in its output.
record_file() {
    identifier=$1
    mode=$2
    destination=$3
    checksum=$(sha256sum < "$destination")
    checksum=${checksum%% *}
    printf 'file\t%s\t%s\t%s\n' "$identifier" "$mode" "$checksum"
}

# Publish the ownership record only after every payload file has been installed.
# A failed publication leaves untracked files and must be reported, not concealed.
umask 077
temporary_manifest=$(mktemp)
{
    printf 'chessvault-install-manifest\t1\n'
    printf 'prefix\t%s\n' "$resolved_prefix"
    printf 'data-home\t%s\n' "$data_home"
    record_file binary 0755 "$binary_destination"
    record_file desktop 0644 "$desktop_destination"
    record_file png 0644 "$png_destination"
    record_file svg 0644 "$svg_destination"
    record_file license 0644 "$license_destination"
} > "$temporary_manifest"
install_file 0600 "$temporary_manifest" "$manifest_destination"

# Cache refresh is optional and must not turn a completed install into failure.
if command -v update-desktop-database > /dev/null 2>&1; then
    run update-desktop-database "$data_home/applications" ||
        printf '%s\n' 'Warning: could not refresh the desktop database.' >&2
fi
if command -v gtk-update-icon-cache > /dev/null 2>&1 &&
    [ -f "$data_home/icons/hicolor/index.theme" ]; then
    run gtk-update-icon-cache -f "$data_home/icons/hicolor" ||
        printf '%s\n' 'Warning: could not refresh the icon cache.' >&2
fi

printf 'Installed ChessVault: %s\n' "$binary_destination"
case ":${PATH:-}:" in
    *":$resolved_prefix/bin:"*) ;;
    *)
        printf 'For terminal launch, add this directory to PATH: %s\n' "$resolved_prefix/bin"
        printf '%s\n' 'The desktop launcher does not require a PATH change.'
        ;;
esac
