# ChessVault

ChessVault is a desktop chess application.

## Requirements

This package is for Linux x86_64 with glibc 2.35 or newer (for example, Ubuntu 22.04 or newer).
A graphical desktop session is required.

## Extract and run

Extract the downloaded tarball into an empty directory, then open a terminal in that directory.
You can run ChessVault without installing it:

```sh
./bin/chessvault
```

## Install for your user

Preview the installation, then install without sudo:

```sh
./install-linux.sh --dry-run
./install-linux.sh
```

By default, the installer places files at:

- `$HOME/.local/bin/chessvault`
- `$HOME/.local/share/applications/chessvault.desktop`
- `$HOME/.local/share/icons/hicolor/256x256/apps/chessvault.png`
- `$HOME/.local/share/icons/hicolor/scalable/apps/chessvault.svg`
- `$HOME/.local/share/licenses/chessvault/LICENSE`

Launch ChessVault from your desktop's application menu or run `chessvault` if `$HOME/.local/bin` is on your `PATH`.
Otherwise, run `"$HOME/.local/bin/chessvault"` directly.

Use `./install-linux.sh --prefix /absolute/path` to choose a different executable installation prefix.
This installs the executable into that prefix's `bin` directory and requires interactive confirmation.
It does not change where the desktop entry and icons are installed. System-wide installation is not supported.

If `XDG_DATA_HOME` is set to an absolute path, the desktop entry, icons, and license go there instead of
`$HOME/.local/share`.
Use the same setting as your desktop session so it can find the launcher and icons.

A successful installation also creates `$XDG_DATA_HOME/chessvault/install-manifest-v1`
(default: `$HOME/.local/share/chessvault/install-manifest-v1`) with owner-only permissions.
It records the executable prefix, data directory, and SHA-256 checksums and permissions of the five installed files.
The desktop entry checksum covers the generated launcher, not the package template.
Reinstall, upgrade, and uninstall use this metadata to verify files before changing them.
Do not edit the manifest. Installation paths must not contain tabs or line breaks.

## Reinstall or upgrade

Extract the desired package into a new directory, then run its installer with the same prefix and
`XDG_DATA_HOME` used for the existing installation:

```sh
./install-linux.sh --dry-run
./install-linux.sh
```

Running again reinstalls the packaged files or upgrades them when the package has changed.
Missing recorded files are restored. Modified files, symlinks, invalid manifests, different prefixes,
and untracked destination files are refused; inspect conflicts and move them aside before retrying.
Installations without a manifest require manual removal before installing again.

After validation, the installer backs up every existing managed file and the manifest, preserving permissions,
before staging replacements. If backup fails, the existing installation is left unchanged.
Backups are temporary and removed on success. If replacement fails, the installer attempts rollback.
If rollback fails, it reports the retained backup directory and affected destinations for manual recovery.
Do not delete retained backups until recovery is complete. Abrupt termination can also require manual recovery.
User state and unrelated files are preserved. Do not run installers concurrently or edit installed files during an operation.

## Remove an installation

Preview removal, then uninstall with the same prefix and `XDG_DATA_HOME` used for installation:

```sh
./install-linux.sh --uninstall --dry-run
./install-linux.sh --uninstall
```

The original package payload is not required, but keep a copy of the installer script.
Uninstall validates the manifest and checks every remaining file's content and permissions before deleting anything.
Modified files, symlinks, malformed manifests, or mismatched installation roots cause removal to stop.
Inspect and move modified files aside before retrying; missing installed files are allowed.
If removal fails partway through, the manifest is retained so you can retry.
User state, logs, unrelated files, and directories are preserved.

Older installations without a manifest require manual removal. For a default installation:

```sh
rm -- "$HOME/.local/bin/chessvault" \
  "$HOME/.local/share/applications/chessvault.desktop" \
  "$HOME/.local/share/icons/hicolor/256x256/apps/chessvault.png" \
  "$HOME/.local/share/icons/hicolor/scalable/apps/chessvault.svg" \
  "$HOME/.local/share/licenses/chessvault/LICENSE" \
  "$HOME/.local/share/chessvault/install-manifest-v1"
```

If you used a custom prefix or `XDG_DATA_HOME`, adjust the paths accordingly.
You can also delete the extracted package when you no longer need it.

## Project and support

- Project: <https://github.com/chessvaultdotapp/chessvault>
- Report issues: <https://github.com/chessvaultdotapp/chessvault/issues>

See `share/licenses/chessvault/LICENSE` in this package for license terms.
