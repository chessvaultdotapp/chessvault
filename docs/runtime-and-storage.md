# Runtime and storage

[Documentation index](index.md) · [Architecture](architecture.md)

Storage responsibilities are split between platform policy, application naming,
and desktop-owned persistence. The runtime package is currently a small path
helper library, not an async executor or background-service framework.

## Ownership and source map

| Layer | Responsibility | Source |
| --- | --- | --- |
| `platform-dirs` | Resolve the user's state-directory base | [`lib.rs`](../crates/platform-dirs/src/lib.rs), [`linux.rs`](../crates/platform-dirs/src/linux.rs) |
| `application-runtime` | Append the application directory name, `chessvault` | [`fs.rs`](../crates/application-runtime/src/fs.rs) |
| Desktop path helper | Append the window-state filename, `recreate` | [`fs.rs`](../apps/chessvault/src/fs.rs) |
| Desktop application | Create directories, serialize, read, and write window state | [`main.rs`](../apps/chessvault/src/main.rs) |

The three path helpers return `anyhow::Result<PathBuf>` and do not create
anything. Paths stay as `PathBuf`/`OsString` values so non-Unicode filesystem
names remain usable. Display formatting is for error messages, not path
reconstruction.

## Choosing the base directory

### Development builds

`platform-dirs` enables its `development` feature by default. When both that
feature and debug assertions are enabled, `user_state_dir()` returns:

```text
<current working directory>/.local/state
```

This branch works on every platform and returns an error if the working
directory cannot be obtained. It does not consult `XDG_STATE_HOME`. The current
runtime dependency uses the default features, so normal debug app builds follow
this policy.

Resolution happens on each call, rather than being cached. Launching from a
different directory therefore selects different development state. Changing the
process working directory during execution would also change later resolutions.

Consumers can request platform behavior in debug builds by disabling default
features on their `platform-dirs` dependency. Cargo features are additive: that
only works if no other dependency enables `development` for the same package.

### Platform builds

Without the development/debug combination, only Linux is currently implemented:

1. Use `XDG_STATE_HOME` if it is an absolute path.
2. Otherwise obtain the home directory through `std::env::home_dir` and require
   it to be absolute.
3. Append `.local/state` to that home directory.
4. Return an error if neither source yields a usable path.

Unset, empty, and relative `XDG_STATE_HOME` values all take the fallback path.
An absolute XDG path does not require a successful home-directory lookup.

On other operating systems, platform resolution returns an unsupported-platform
error. Debug builds using the development path may run there, but release
builds currently cannot complete the app's state-directory initialization.

## Application paths

`application_runtime::fs::application_state_dir()` appends `chessvault` to the
base directory. The desktop helper then appends `recreate`:

| Configuration | Window-state file |
| --- | --- |
| Default debug build, launched from workspace root | `<workspace>/.local/state/chessvault/recreate` |
| Linux platform resolution with absolute XDG state path | `<XDG_STATE_HOME>/chessvault/recreate` |
| Linux platform resolution using home fallback | `<home>/.local/state/chessvault/recreate` |

The desktop creates the application directory with `create_dir_all` during
boot. A helper successfully returning a path does not guarantee that the path
exists or is writable.

## Window-state format and lifecycle

The file is named `recreate`, with no extension, but its content is JSON:

```json
{"width":820.5,"height":620.0}
```

Only window dimensions are persisted. Window position, chess position, console
history, and source-toggle settings are not saved.

At startup, the app attempts to read this file before boot creates the state
directory. A missing file is normal and uses Iced's default window size. Other
read errors, invalid JSON, missing required fields, and invalid dimensions are
reported through tracing and also fall back to the default size. Dimensions
must be finite and strictly positive.

During boot, inability to resolve or create the application directory is fatal;
the application logs the error and exits with status 1.

On a window-close request, the app obtains the current size, serializes it,
creates or truncates the file, writes the JSON, and closes the window. A save
failure is logged but does not prevent closing. Writes are synchronous and
currently do not use a temporary file, atomic replacement, or an explicit disk
sync. Interrupted writes can therefore leave invalid state for the next launch,
which the startup fallback handles.

## Error handling and tests

Each layer adds context without discarding the original error. This allows
callers and tests to inspect the underlying I/O or serialization error while
still reporting which path or operation failed.

Resolver tests inject environment values, home-directory lookups, or upstream
path results. Preserve that pattern instead of changing process-wide environment
variables in tests. Linux tests cover XDG precedence, invalid inputs, fallback
errors, and non-Unicode paths. Application-runtime and desktop path tests cover
name composition, error preservation, and non-Unicode paths where supported.
Window-state parsing tests cover round trips and rejected dimensions.

Use the ordered checks in [CONTRIBUTING.md](../CONTRIBUTING.md). Package-local
checks can select `-p platform-dirs` or `-p application-runtime`; changes spanning
the path pipeline should use workspace checks. Default debug tests do not alone
exercise every compiled platform branch. For platform-directory changes, also
run the ordered package checks with `--no-default-features`, keeping the fmt
command unchanged. Native directory support still needs verification on the
corresponding operating system.

## Troubleshooting

- **Window size is not restored:** confirm the working directory and build mode;
  they determine which state file is read. Check startup logs for read or parse
  errors.
- **XDG settings appear ignored:** the default debug development path takes
  precedence over Linux platform resolution.
- **The app fails during initialization:** check directory permissions and path
  resolution. Non-Linux platform builds currently lack a native resolver.
- **Resetting window state:** with the app closed, remove the resolved `recreate`
  file. Removing it while the app is open is temporary because closing writes
  the current dimensions again.
