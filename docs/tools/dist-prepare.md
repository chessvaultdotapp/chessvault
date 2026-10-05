# Distribution preparation tool

[`tools/dist-prepare/main.py`](../../tools/dist-prepare/main.py) stages a built
ChessVault binary and the repository's desktop entry in a directory ready for
creating a native release tarball. It does not build binaries, create archives,
install files, or publish releases.

Commands follow our [shell command conventions](../reference.md#shell-commands).
Run the examples from the workspace root. Output is illustrative; repository
paths and test timings vary.

## Setup

Install [uv](uv.md) and [just](just.md), then create the dedicated environment:

```console
$ just --justfile venvs/justfile dist-prepare
uv venv dist-prepare
...
uv pip install --python dist-prepare/bin/python --requirements dist-prepare.requirements.txt
...
```

The recipe runs inside `venvs/` and creates the Git-ignored
`venvs/dist-prepare/` environment. Its pinned requirements provide pytest for
tests; staging itself uses only the Python standard library.
See [Python environment recipes](just.md#python-environments) for Python version
selection. These recipes use the POSIX environment layout; on Windows, run the
uv commands manually with `Scripts/python.exe` instead of `bin/python`.
Likewise, use `venvs/dist-prepare/Scripts/python.exe` for the examples below.

## Prepare a package

Build the release binary with the repository's pinned [Rust toolchain](rust.md),
or supply an already-built binary. Place the generated `chessvault.png` beside it.
`build.rs` writes this 256×256 PNG into its Cargo `OUT_DIR`; use the `out_dir`
from the matching `build-script-executed` message with `--message-format=json`
to locate it reliably (as the release workflow does). Below, replace `<OUT_DIR>`
with that directory:

```console
$ cargo build -p chessvault --bin chessvault --release --locked
...
$ cp <OUT_DIR>/chessvault.png target/release/chessvault.png
$ venvs/dist-prepare/bin/python tools/dist-prepare/main.py target/release/chessvault
Prepared /path/to/chessvault/dist/package
```

The binary path is required. The default output is `dist/package/` under the
repository containing the tool, regardless of the current working directory.
Use `--output` to select a different, not-yet-existing staging directory:

```console
$ venvs/dist-prepare/bin/python tools/dist-prepare/main.py target/release/chessvault --output dist/native
Prepared dist/native
```

Explicit relative paths are resolved from the current working directory.
The desktop entry is always read from
[`apps/chessvault/data/chessvault.desktop`](../../apps/chessvault/data/chessvault.desktop)
in the tool's repository, not from the current directory.

The resulting package contains:

```text
dist/package/
├── bin/
│   └── chessvault
└── share/
    ├── applications/
    │   └── chessvault.desktop
    └── icons/hicolor/256x256/apps/
        └── chessvault.png
```

The binary is copied with mode `0755`; the desktop entry and PNG use `0644`.
Source contents and permissions are not changed. No libraries or additional
assets are bundled, and the tool does not validate the binary's architecture
or runtime dependencies. The executable needs compatible system libraries.

## Tarball creation and CI

Archive creation is a separate step. For example, after staging into the default
output directory:

```console
$ tar -czvf dist/chessvault-linux-x86_64.tar.gz -C dist/package bin share
bin/
bin/chessvault
share/
share/applications/
share/applications/chessvault.desktop
share/icons/
share/icons/hicolor/
share/icons/hicolor/256x256/
share/icons/hicolor/256x256/apps/
share/icons/hicolor/256x256/apps/chessvault.png
```

The [development release workflow](../../.github/workflows/development-release.yml)
checks out the tagged commit and downloads the shared Linux binary and generated PNG, then runs:

```console
$ python3 tools/dist-prepare/main.py binary/chessvault --output dist/package
Prepared dist/package
```

CI uses the runner's Python without installing test dependencies. It archives
`bin/` and `share/` as `chessvault-<dev-tag>-linux-x86_64.tar.gz`.
[AppImage packaging](../development/appimage.md) runs separately using the same
built binary; it does not use this staging tool. See the
[release tag tool](tag-tool.md#publish-on-github) for publication details.

## Failures and recovery

The tool rejects a missing or non-file binary or sibling `chessvault.png`, and any existing output path,
including directories and dangling symlinks. It never merges into or replaces
an existing package directory. Review an old package before removing it, or
choose a new `--output` path when retrying.

Files are copied into a temporary sibling directory before the completed package
is moved into place. Copy failures, including a missing desktop entry, clean up
the temporary staging directory without leaving a partial package. Newly created
parent directories may remain. Other artifacts under `dist/` are left untouched.
Errors are printed to standard error and return exit status 1.

## Tests

```console
$ venvs/dist-prepare/bin/python -m pytest tools/dist-prepare -q
...
10 passed in 0.10s
```

Tests cover package layout, file contents and permissions, preservation of other
artifacts and existing output, invalid binaries, cleanup after copy failures,
and CLI execution from another directory. They use temporary directories and
fake binary contents; no Rust build or desktop launch is required.

For linting, formatting, and type checks, install the
[shared Python development environment](index.md#shared-python-development-environment):

```console
$ venvs/development/bin/ruff check tools/dist-prepare
All checks passed!
$ venvs/development/bin/ruff format --check tools/dist-prepare
2 files already formatted
$ venvs/development/bin/ty check --python venvs/dist-prepare tools/dist-prepare
All checks passed!
```
