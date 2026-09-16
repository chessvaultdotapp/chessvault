# Application runtime

`application-runtime` provides application-specific runtime helpers. It currently
resolves Chessvault's state-directory path; it is not an async executor or a
background-service framework. It depends on `platform-dirs`, not the GUI or chess
core.

## Source map

| File | Responsibility |
| --- | --- |
| [`lib.rs`](../../crates/application-runtime/src/lib.rs) | Exposes the public `fs` module |
| [`fs.rs`](../../crates/application-runtime/src/fs.rs) | Application state-directory resolution and tests |

## State-directory policy

`application_runtime::fs::application_state_dir()` returns an
`anyhow::Result<PathBuf>`. It calls `platform_dirs::user_state_dir()` and appends
`chessvault` to the result. It does not create the directory or guarantee that
it exists or is writable.

| Base-directory policy | Application state directory |
| --- | --- |
| Default debug build | `<current working directory>/.local/state/chessvault` |
| Linux with absolute `XDG_STATE_HOME` | `<XDG_STATE_HOME>/chessvault` |
| Linux home fallback | `<home>/.local/state/chessvault` |

The runtime dependency enables the default `platform-dirs` features. Debug builds
therefore use working-directory-local state even when `XDG_STATE_HOME` is set.
See [Platform directories](platform-dirs.md) for the feature conditions and
supported platforms.

Paths remain `PathBuf` values, preserving non-Unicode names. Resolution errors
receive application-level context without discarding their underlying cause.

## Boundary with the desktop

The desktop creates the application directory during boot. Its own
[`fs.rs`](../../apps/chessvault/src/fs.rs) appends `recreate` for saved window
state; the runtime library does not choose that filename or define its format.

When launched from the workspace root with the default debug configuration,
the desktop writes `.local/state/chessvault/recreate`. The file contains JSON
width and height values, not a saved chess position. See
[Desktop application](desktop.md#window-persistence-and-failures) for loading,
saving, and failure behavior.

Keep application naming policy here, platform discovery in `platform-dirs`, and
consumer-specific persistence with the consumer. Returning a path should not
silently introduce filesystem writes.

## Verification

Inline tests inject an upstream path result to cover application-name
composition, contextual errors, and non-Unicode paths on Unix. They do not need
to modify process-wide environment variables or create directories.

Follow [Testing](../development/testing.md), selecting `-p application-runtime`
for package-local changes. Use workspace checks when changing the path contract
across the runtime, platform resolver, and desktop.
