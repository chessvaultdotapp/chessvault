# just

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[just](https://just.systems/) provides shortcuts for common development commands
in the root [`justfile`](../../justfile). It is optional; the Cargo commands
throughout these docs remain usable directly. Rust CI invokes Cargo directly;
the platform-dirs documentation workflow uses just for diagram generation.

Install it through Cargo, or follow the
[official installation instructions](https://just.systems/man/en/packages.html):

```console
$ cargo install just --locked
    Updating crates.io index
...
$ just --version
just 1.58.0
```

After cloning, run `just` from the workspace root to list available tasks:

```console
$ just
Available recipes:
...
```

| Command | Purpose |
| --- | --- |
| `just check` | Build, test, check formatting, and lint the workspace. |
| `just check-package chess-core` | Run checks for one package; formatting stays workspace-wide. |
| `just check-platform-native` | Check `platform-dirs` with default features disabled. |
| `just run` | Launch the desktop application. |
| `just run-debug` | Launch with `RUST_LOG=chessvault=debug`. |
| `just fmt` | Apply Rust formatting across the workspace. |
| `just optimize-assets` | Optimize desktop SVG artwork in place, recursively. |

Check recipes follow the [ordered verification sequence](../development/testing.md#ordered-checks),
stop on failure, and require [cargo-nextest](cargo-nextest.md). They run
doctests separately, except for the binary-only `chessvault` package.
`check-package` also accepts `chessvault`, `application-runtime`, and
`platform-dirs`. Native path checks supplement the default checks; see
[platform and feature variants](../development/testing.md#platform-and-feature-variants).
If nextest is unavailable, use the manual Cargo fallback described in the
[testing guide](../development/testing.md#ordered-checks).

`just optimize-assets` checks that Node.js and pnpm are available. Install the
[asset-tooling dependencies](svgo.md) first; the recipe does not
install them automatically. Optimization is manual and modifies SVGs in place;
review the diff and follow the [visual verification workflow](../components/desktop.md#optimizing-svg-artwork).

## Python environments

The separate [`venvs/justfile`](../../venvs/justfile) uses [uv](uv.md) to create
Python environments and install their pinned requirements. From the workspace
root, select it explicitly:

```console
$ just --justfile venvs/justfile --list
Available recipes:
    default
    development
    dist-prepare
    tag-tool
    venvs
    zensical
```

| Command | Purpose |
| --- | --- |
| `just --justfile venvs/justfile development` | Set up Ruff and ty. |
| `just --justfile venvs/justfile dist-prepare` | Set up the distribution preparation tool's pytest environment. |
| `just --justfile venvs/justfile tag-tool` | Set up the release tag tool and pytest. |
| `just --justfile venvs/justfile zensical` | Set up the documentation-site dependencies. |
| `just --justfile venvs/justfile venvs` | Set up all virtual environments. |

Recipes run relative to `venvs/`, not the calling shell's directory. Each runs
`uv venv <name>` followed by installation from `<name>.requirements.txt`.
They are setup recipes, not test commands, and may prompt before replacing an
existing environment.

The recipes do not pin a Python version. To select Python 3.14 explicitly, set
uv's `UV_PYTHON` environment variable:

```console
$ UV_PYTHON=3.14 just --justfile venvs/justfile venvs
uv venv development
...
```

The current Python CI workflow uses Python 3.14.5. The tag tool requires
Python 3.11 or newer for `datetime.UTC`; installed tools may have their own
minimum versions. Recipes currently use POSIX `bin/python` paths. On Windows,
run the equivalent uv commands with `Scripts/python.exe` instead.

See the [shared development environment](index.md#shared-python-development-environment),
[tag tool setup](tag-tool.md#setup), and
[distribution preparation setup](dist-prepare.md#setup) for usage after installation.
