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

The installer currently refuses to overwrite existing files, including files from an earlier ChessVault installation.
Remove those files before installing a newer version.

## Remove an installation

The installer's `--uninstall` option is not implemented yet. For a default installation, remove these files manually:

```sh
rm -- "$HOME/.local/bin/chessvault" \
  "$HOME/.local/share/applications/chessvault.desktop" \
  "$HOME/.local/share/icons/hicolor/256x256/apps/chessvault.png" \
  "$HOME/.local/share/icons/hicolor/scalable/apps/chessvault.svg" \
  "$HOME/.local/share/licenses/chessvault/LICENSE"
```

If you used a custom prefix or `XDG_DATA_HOME`, adjust the paths accordingly.
You can also delete the extracted package when you no longer need it.

## Project and support

- Project: <https://github.com/chessvaultdotapp/chessvault>
- Report issues: <https://github.com/chessvaultdotapp/chessvault/issues>

See `share/licenses/chessvault/LICENSE` in this package for license terms.
