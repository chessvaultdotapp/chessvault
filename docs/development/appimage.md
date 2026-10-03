# Local AppImage packaging

Build on Linux x86_64 with the repository's pinned Rust toolchain, `curl`,
`sha256sum` (coreutils), and `desktop-file-validate` (desktop-file-utils).
The first build needs network access for Cargo dependencies and packaging tools.

From the workspace root:

```console
$ just appimage
...
Built /path/to/chessvault/dist/ChessVault-x86_64.AppImage
```

Without Just, run `bash tools/build-appimage.sh`. The script also works from
outside the repository. It builds the desktop in release mode with the locked
dependencies and stages the executable, desktop entry, and icon in a fresh AppDir.
Linuxdeploy bundles eligible linked libraries and generates the launcher and image.

Linuxdeploy (including its AppImage output plugin) and the AppImage runtime use
fixed releases with checked SHA-256 hashes. Downloads are cached under
`target/appimage-tools/`. If checksum validation fails, remove the affected cached
file and retry; do not bypass validation. Temporary staging directories are removed
on exit. The output in `dist/` is overwritten on subsequent successful builds.

## Run and verify

<!-- Release GUI launches are silent on success. -->
<!-- rumdl-disable MD014 -->

Launch from outside the checkout to verify directory-independent assets:

```console
$ cd /tmp
$ /path/to/chessvault/dist/ChessVault-x86_64.AppImage
```

If FUSE is unavailable, use extraction mode (also used by the packaging script):

```console
$ APPIMAGE_EXTRACT_AND_RUN=1 /path/to/chessvault/dist/ChessVault-x86_64.AppImage
```

<!-- rumdl-enable MD014 -->

Check board rendering, legal moves, resizing, the F12 console, and closing/reopening
to restore window dimensions. Release state is written beneath
`$XDG_STATE_HOME/chessvault`, or `~/.local/state/chessvault` when unset, rather than
inside the image or checkout.

## Scope and limitations

This is a native local build, not yet a portable release pipeline. AppImage does
not remove the host glibc baseline requirement. Build on an older supported Linux
baseline before distributing to older systems, then test on other distributions.
Graphics drivers and dynamically loaded Wayland/X11, Vulkan/OpenGL, and font
libraries can still depend on the host; a successful packaging run alone does not
establish compatibility. Test both Wayland and X11 environments.

There is no cross-compilation, signing, update integration, AppStream metadata,
or automatic desktop installation yet. The rook icon is a basic packaging icon.
