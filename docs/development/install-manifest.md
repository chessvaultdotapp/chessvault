# Linux installation manifest

The native Linux installer writes an installation manifest after installing its five payload files.
The manifest records their destinations indirectly through installation roots and fixed file identifiers,
plus their content checksums and installed permission modes. Uninstall uses this record without
requiring the original package payload.

**Current status:** manifest creation, manifest-based uninstall, reinstall, and upgrade are implemented.
Untracked destination files and invalid or modified installations cause installation to fail.

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

Checksums are computed with `sha256sum` from staged file contents that are copied into the installation.
In particular, the desktop entry digest covers the generated launcher with its absolute executable path,
not the packaged template.
Modes record the permissions requested by the installer, not a subsequent filesystem inspection.
The manifest does not record itself, directories, ownership IDs, timestamps, or the application version.

## Creation and failures

The installer checks that `sha256sum` is available. For a fresh installation, all six destinations
must be absent. For reinstall or upgrade, it first validates the complete existing manifest and files
using the same rules as uninstall. Missing recorded files are allowed; changing roots is not.

Before staging replacement payload, the installer copies every existing managed file and the manifest
with permissions preserved into a private temporary directory. Backup failure stops the operation
without changing the installation. The backup location is printed; backups are removed on success.
No application state or unrelated files are included in these backups.

The complete payload and new manifest are then staged. The existing installation is validated again
before replacement. Each file is installed into a temporary sibling and renamed into place, with
the manifest published last. This avoids truncating a running executable. The manifest format is
unchanged by reinstall or upgrade.

On command failure or a handled signal, the installer attempts rollback of replaced files in reverse
order, restoring the backed-up content and modes and removing newly created files. If restoration
fails, remaining backups are retained and the installer reports their location and affected destinations.
Backup filenames `old-binary`, `old-desktop`, `old-png`, `old-svg`, `old-license`, and `old-manifest`
map to the destinations above; restore remaining files to those destinations before retrying.
Do not discard retained backups before completing recovery.

`--dry-run` validates the existing installation and reports destinations without creating backups,
staging files, or changing the installation. There is no whole-installation atomicity or concurrency
lock. Abrupt termination, such as SIGKILL or power loss, can leave a partial installation requiring
manual recovery from the printed backup directory. A manifest's presence alone is not proof of a
complete or unchanged installation. Fresh-install failures can leave newly created empty directories.

## Uninstall validation

`--uninstall` requires a regular manifest and matching prefix and data-home settings. It validates
all records in order and all remaining destinations before deleting any files. Symlinked parent
directories, destination symlinks, non-regular files, changed content, and changed permission modes
are rejected. Missing payload files are allowed, but a missing manifest is an error, including on
a repeated uninstall after successful removal.

`--uninstall --dry-run` performs the same validation and prints planned removals without changes.
After successful validation, uninstall removes the remaining payload files, then the manifest.
A removal failure retains the manifest for retry. Shared directories, unrelated files, and application
state are left untouched. Optional desktop/icon cache refresh failures are warnings.

Validation and removal are not an atomic operation and there is no concurrency lock; do not run
multiple installers or modify the installed files during uninstall.

The uninstall and upgrade reader follows these safety rules:

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

The writer and uninstall reader are in [`scripts/install-linux.sh`](../../scripts/install-linux.sh).
Fixture tests in [`scripts/tests/test_install_linux.py`](../../scripts/tests/test_install_linux.py)
verify the exact records, checksums, permissions, custom roots, delimiter rejection, and preservation
of existing manifests. They also cover uninstall without payload, dry runs, malformed manifests,
modified files, symlinks, missing files, partial removal/retry, and state preservation.
Reinstall and upgrade tests cover backup-before-replacement, custom roots, missing-file restoration,
backup and replacement failures, rollback, and retained recovery backups. Real-package tests in
[`scripts/package-tests/test_package_install.py`](../../scripts/package-tests/test_package_install.py)
use independent scenarios for package contents, installation dry-run, installation and manifest creation,
reinstall, upgrade, and uninstall. Each lifecycle scenario runs with fresh default and custom XDG data
locations; only package-content validation is independent of installation paths.

See [Testing](testing.md) for fixture and built-package test commands, and
[Distribution preparation](../tools/dist-prepare.md) for package staging.
