# Linux installation manifest

The native Linux installer writes an installation manifest after installing its five payload files.
The manifest records their destinations indirectly through installation roots and fixed file identifiers,
plus their content checksums and installed permission modes. It is intended to support safe uninstall
and upgrade operations without requiring the original package.

**Current status:** manifest creation is implemented; manifest reading, automatic uninstall, and upgrades
are not. Existing files, including an existing manifest, still cause installation to fail.

## Location and lifetime

The manifest is stored at `<data-home>/chessvault/install-manifest-v1` with mode `0600`
(owner read/write only). The data-home is an absolute `XDG_DATA_HOME`, or `$HOME/.local/share`
when that variable is unset, empty, or relative.

`--prefix` changes the executable location, not the manifest location. The manifest shares the
same data-home as the desktop entry, icons, and license. Consequently, separate executable prefixes
using the same data-home cannot have independent manifests or desktop integration.

The manifest is installer metadata, not application state. It contains no logs, preferences,
window dimensions, or other user data. Users should not edit it. Older installations created
without a manifest are not automatically adopted.

## Version 1 format

The file is tab-separated, with one record per line and a final newline. The following example
uses the visible characters `\t` to represent a single literal tab; `<sha256>` stands for a
64-character lowercase hexadecimal SHA-256 digest:

```text
chessvault-install-manifest\t1
prefix\t/home/alice/.local
data-home\t/home/alice/.local/share
file\tbinary\t0755\t<sha256>
file\tdesktop\t0644\t<sha256>
file\tpng\t0644\t<sha256>
file\tsvg\t0644\t<sha256>
file\tlicense\t0644\t<sha256>
```

The writer emits exactly these eight records in this order:

| Record | Fields | Meaning |
| --- | --- | --- |
| Header | `chessvault-install-manifest`, `1` | Format identifier and version |
| Prefix | `prefix`, absolute path | Executable prefix resolved by `realpath -m` |
| Data home | `data-home`, absolute path | Selected data-home, recorded without canonicalization |
| File (five records) | `file`, identifier, mode, digest | Fixed file identity, octal permissions, content hash |

Paths are written literally, without quoting or backslash escaping. Spaces, backslashes, and
non-Unicode path bytes are preserved; consumers must not assume UTF-8. Tabs, carriage returns,
and newlines in either installation root are rejected before installation to avoid ambiguous records.

## File identifiers and destinations

The manifest does not contain arbitrary per-file paths. Derive each destination from its fixed
identifier and the recorded roots:

| Identifier | Destination | Mode |
| --- | --- | --- |
| `binary` | `<prefix>/bin/chessvault` | `0755` |
| `desktop` | `<data-home>/applications/chessvault.desktop` | `0644` |
| `png` | `<data-home>/icons/hicolor/256x256/apps/chessvault.png` | `0644` |
| `svg` | `<data-home>/icons/hicolor/scalable/apps/chessvault.svg` | `0644` |
| `license` | `<data-home>/licenses/chessvault/LICENSE` | `0644` |

Checksums are computed with `sha256sum` from installed file contents. In particular, the desktop
entry digest covers the generated launcher with its absolute executable path, not the packaged template.
Modes record the permissions requested by the installer, not a subsequent filesystem inspection.
The manifest does not record itself, directories, ownership IDs, timestamps, or the application version.

## Creation and failures

The installer checks that `sha256sum` is available and that none of the five payload destinations
or the manifest destination already exists, including dangling symlinks. It then installs the payload,
hashes the installed files, and writes a temporary manifest before copying it to its final destination.
The final manifest is installed only after all five payload files have been installed.

`--dry-run` reports the manifest destination but does not create a manifest or calculate its checksums.
An installation failure can leave partially installed files, a missing manifest, or an incomplete
manifest destination. There is no transactional rollback or whole-installation atomicity. A manifest's
presence alone is therefore not proof of a complete or unchanged installation.

## Requirements for future readers

These are safety requirements for subsequent uninstall and upgrade work, not current behavior:

- Parse the manifest as data; never source it or evaluate its contents as shell commands.
- Reject unsupported versions, malformed records, invalid hashes or modes, duplicate or missing identifiers,
  and unexpected records before changing files.
- Validate the roots and derive only the five permitted destinations; a manifest is not authorization
  to delete arbitrary paths.
- Verify current file types, content hashes, and modes before treating files as unchanged installed files.
  Do not silently follow destination symlinks or delete modified or unrelated files.
- Treat missing or malformed manifests conservatively; do not infer ownership from filenames alone.
- Preserve application state and unrelated files. Checksums are an accidental-change safeguard, not
  protection against a malicious process running as the same user.

## Implementation and verification

The writer is in [`scripts/install-linux.sh`](../../scripts/install-linux.sh).
Fixture tests in [`scripts/tests/test_install_linux.py`](../../scripts/tests/test_install_linux.py)
verify the exact records, checksums, permissions, custom roots, delimiter rejection, and preservation
of existing manifests. Real-package tests in
[`scripts/package-tests/test_package_install.py`](../../scripts/package-tests/test_package_install.py)
also verify manifest contents and that a refused reinstall leaves it unchanged.

See [Testing](testing.md) for fixture and built-package test commands, and
[Distribution preparation](../tools/dist-prepare.md) for package staging.
