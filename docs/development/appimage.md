# AppImage packaging and distribution

Build on Linux x86_64 with the repository's pinned Rust toolchain, `curl`,
`sha256sum` (coreutils), `readelf` (binutils), and `desktop-file-validate` (desktop-file-utils).
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

To package an existing release binary without invoking or requiring Cargo, use
`bash tools/build-appimage.sh --binary /path/to/chessvault`. Relative paths are
resolved from the calling directory. The file must be executable and built for
Linux x86_64 with a compatible distribution baseline. CI uses this mode so the
native archive and AppImage share a single compilation.

Linuxdeploy (including its AppImage output plugin) and the AppImage runtime use
fixed releases with checked SHA-256 hashes. Downloads are cached under
`target/appimage-tools/`. If checksum validation fails, remove the affected cached
file and retry; do not bypass validation. Temporary staging directories are removed
on exit. The image is extracted and its entry point, desktop entry, and bundled
project license are checked before replacing the output in `dist/`. Each build also
writes a `.glibc.txt` inventory of required glibc versions.

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

## Distribution

Development prereleases include a versioned AppImage. GitHub provides a SHA-256
digest for each release asset; no separate checksum file is shipped. Compare the
output of `sha256sum 'chessvault-<dev-tag>-linux-x86_64.AppImage'` with the asset's
SHA-256 digest on GitHub before allowing execution:

<!-- chmod and successful GUI launches are silent. -->
<!-- rumdl-disable MD014 -->

```console
$ chmod +x 'chessvault-<dev-tag>-linux-x86_64.AppImage'
$ './chessvault-<dev-tag>-linux-x86_64.AppImage'
```

<!-- rumdl-enable MD014 -->

Replace `<dev-tag>` with the release tag. Checksums detect corruption; they are not
signatures. Only download artifacts from the project's trusted release page.
Desktop integration is optional and is not performed automatically.

Release CI builds on Ubuntu 22.04 and rejects packaged binaries requiring glibc
newer than 2.35. Local builds inherit the local system baseline and must not be
substituted for release builds on newer hosts. Ubuntu 22.04 or newer is the initial
compatibility target, not yet a verified support guarantee for every distribution.

## Release acceptance checklist

Before advertising general Linux support, test the **published image** on clean
Ubuntu 22.04 and a current Fedora installation, with both X11 and Wayland:

- Compare its SHA-256 checksum with GitHub's asset digest and launch outside the checkout.
- Test native FUSE launch and `APPIMAGE_EXTRACT_AND_RUN=1` without FUSE.
- Exercise the board, legal moves, F12 console, resizing, and window-state restoration.
- Check hardware rendering and software rendering in a VM without a development toolchain.
- Record distribution, graphics stack, session type, artifact checksum, and results in release notes.
- Review redistribution licenses for bundled dependencies before a public release.

CI validates packaging, not these interactive compatibility checks.

## Scope and limitations

AppImage does not remove the host glibc baseline requirement.
Graphics drivers and dynamically loaded Wayland/X11, Vulkan/OpenGL, and font
libraries can still depend on the host; a successful packaging run alone does not
establish compatibility. Test both Wayland and X11 environments.

There is no cross-compilation, signing, update integration, AppStream metadata,
or automatic desktop installation yet. The rook icon is a basic packaging icon.
