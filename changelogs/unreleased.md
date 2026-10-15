# Unreleased

## Added

- Added a per-user Linux installer to native release tarballs. Run `./install-linux.sh` to install the executable,
  application-menu launcher, icons, and license without sudo. Supports installation previews with `--dry-run`,
  a custom executable prefix with `--prefix`, and `XDG_DATA_HOME` for desktop integration files.
  Existing files are not overwritten; upgrades require manually removing the previous installation, and automated
  uninstall is not yet available.

- Added a README and license to Linux distribution packages, with instructions for running, installing, upgrading,
  and manually removing the application.
