# Unreleased

## Added

- Added a per-user Linux installer to native release tarballs. Run `./install-linux.sh` to install the executable,
  application-menu launcher, icons, and license without sudo. Supports installation previews with `--dry-run`,
  a custom executable prefix with `--prefix`, and `XDG_DATA_HOME` for desktop integration files.
  Running the installer again reinstalls or upgrades a recorded installation, backing up existing files before
  replacement and attempting rollback on failure. Modified or untracked files are not overwritten.
  Use `--uninstall` to remove a recorded installation while preserving user state and unrelated files;
  removal also supports `--dry-run`. Installations made without a manifest still require manual removal.

- Added a README and license to Linux distribution packages, with instructions for running, installing, upgrading,
  and uninstalling the application, including manual removal of older installations without a manifest.
