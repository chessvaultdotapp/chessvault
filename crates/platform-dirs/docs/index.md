# Platform directories

`platform-dirs` resolves directories for persistent user state, shared data, and logs. It only
resolves paths, optionally appending an application name; callers create directories
as needed.

## API

The crate exposes `platform_dirs::user_state_dir(app_name: Option<&str>)`, returning
`anyhow::Result<PathBuf>`. Pass `None` for the base or `Some("my-app")` for an
application subdirectory. Empty names, `.`, `..`, slashes, backslashes, colons,
and NUL bytes are rejected. See the [in-code documentation](../src/lib.rs) for
resolution rules, errors, and usage examples.

### Log directory

`platform_dirs::user_log_dir(app_name: Option<&str>, opinion: Option<bool>)` returns
`anyhow::Result<PathBuf>`. It calls `user_state_dir(app_name)` and optionally appends
`log`:

| `opinion` | Result |
| --- | --- |
| `None` | State directory plus `log` (default) |
| `Some(true)` | State directory plus `log` |
| `Some(false)` | State directory unchanged |

Rust requires both arguments; pass `None` explicitly to select the default.
For example, `user_log_dir(Some("my-app"), None)` resolves to
`<current working directory>/.local/state/my-app/log` in development mode.
With no application name, `user_log_dir(None, None)` resolves to the base state
directory plus `log`.

Application-name validation, resolution errors, and platform support are identical
to `user_state_dir`. Neither function creates directories.

### Share directory

`platform_dirs::user_share_dir(app_name: Option<&str>)` returns the persistent user
data directory. It uses `<current working directory>/.local/share` in development.
Native Linux resolution prefers an absolute `XDG_DATA_HOME`, falling back to
`.local/share` beneath an absolute home directory. Missing, empty, or relative
XDG values use the fallback. Other native platforms return an error.

Application-name validation and resolution precedence match `user_state_dir`.
Non-Unicode paths are preserved; the function does not create directories.

## Features and platform support

With debug assertions and the default-enabled `development` feature, resolution
uses `<current working directory>/.local/state` for state and
`<current working directory>/.local/share` for shared data on every platform.

Otherwise, native resolution is used. Currently, only Linux is supported:
an absolute `XDG_STATE_HOME` (state) or `XDG_DATA_HOME` (share) takes precedence,
with a fallback to `.local/state` or `.local/share`, respectively, beneath an
absolute home directory. Other platforms return an error.

Resolution preserves non-Unicode paths and does not check whether the directory
exists or is writable.

## Development tasks

Run these tasks from `crates/platform-dirs` using the crate's [justfile](../justfile):

- `just check`: run the workspace-defined checks for this package.
- `just check-native`: check with default features disabled.
- `just diagram`: regenerate the documentation SVGs with Mermaid CLI,
  then optimize them with SVGO. Requires Node.js, pnpm, and the shared tooling dependencies
  installed with `pnpm install --frozen-lockfile` at the workspace root.
- `just diagram-generate`: generate the SVGs without optimizing them.
- `just diagram-optimize`: optimize the existing SVGs without regenerating them.

The `diagram` task runs `diagram-generate` followed by `diagram-optimize`.

Rust commands are delegated to the workspace root. Both check tasks require
cargo-nextest.

## Further reading

- [Architecture](architecture.md): boundaries, module structure, resolution flow,
  and testing strategy
- [Workspace component guide](../../../docs/components/platform-dirs.md)
- [Testing and feature variants](../../../docs/development/testing.md#platform-and-feature-variants)
