# Platform directories

`platform-dirs` resolves user state, share, and log directories, optionally appending
an application name. The desktop passes `Some("chessvault")` and creates the
directory; the library only resolves paths.

## Source map

| File | Responsibility |
| --- | --- |
| [`lib.rs`](../../crates/platform-dirs/src/lib.rs) | Public API, development override, and platform selection |
| [`linux.rs`](../../crates/platform-dirs/src/linux.rs) | Linux resolution and injected-input tests |
| [`macos.rs`](../../crates/platform-dirs/src/macos.rs) | macOS resolution and injected-input tests |

## Public API

`platform_dirs::user_state_dir(app_name: Option<&str>)` returns an `anyhow::Result<PathBuf>`.
Pass `None` for the base or `Some("chessvault")` for its application subdirectory.
Names must be a single nonempty component; `.`, `..`, slashes, backslashes,
colons, and NUL bytes are rejected on every platform. A
successful result identifies a path, not a guarantee that it exists or is
writable. Non-Unicode paths are preserved; display strings should not be used
to reconstruct paths.

### Log directory

`platform_dirs::user_log_dir(app_name: Option<&str>, opinion: Option<bool>)` returns
`anyhow::Result<PathBuf>`. It delegates to `user_state_dir(app_name)` and appends
`log` for `None` or `Some(true)`. `Some(false)` returns the state directory unchanged.
Native macOS instead uses `~/Library/Logs/<app>` regardless of opinion or XDG overrides.
Both arguments must be supplied explicitly in Rust; `None` selects the default opinion.

For example, `user_log_dir(Some("chessvault"), None)` returns
`<current working directory>/.local/state/chessvault/log` under the development override.
Application-name validation and resolution errors are inherited from `user_state_dir`.
The function only resolves the path; it does not create the directory or write logs.

### Share directory

`platform_dirs::user_share_dir(app_name: Option<&str>)` returns an `anyhow::Result<PathBuf>`
for persistent user data, with the same application-name validation as `user_state_dir`.
Both functions resolve the base before validating the name, so resolution errors take precedence.
They preserve non-Unicode paths and do not create directories.

### System share directories

`platform_dirs::system_share_dirs(app_name: Option<&str>)` returns an `anyhow::Result<Vec<PathBuf>>`.
On Linux it reads `XDG_DATA_DIRS` in search order, defaulting to `/usr/local/share` then `/usr/share`
when unset or empty. Relative and empty entries are ignored; an explicitly configured list with no
absolute entries produces an empty list. Order, duplicates, and non-Unicode paths are preserved.
macOS uses the same XDG policy, defaulting instead to `/Library/Application Support`.
The user share directory is not included.

An optional application name is validated using the same rules as user directories and appended to
all returned paths. Invalid names are rejected even for an empty search list. Paths are not created
or checked for existence. Unsupported platforms return an error.

System paths always use native resolution, including in development builds.

### Development override

With both debug assertions and the default-enabled `development` feature, the
base directories are:

```text
<current working directory>/.local/state  (state)
<current working directory>/.local/share  (share)
```

This works on every platform and does not consult `XDG_STATE_HOME` or `XDG_DATA_HOME`. An
unavailable working directory produces an error. The path is resolved on each
call, not cached, so changing the working directory changes later resolutions.

Consumers can disable default features on their dependency to use platform
resolution in debug builds. Cargo features are additive: another dependency
enabling `development` for the same package can re-enable the override.

### Native platform resolution

Release builds, and builds without `development`, use native resolution on Linux and macOS.
Both platforms apply the following policy:

1. Accept `XDG_STATE_HOME` (state) or `XDG_DATA_HOME` (share) when it is absolute.
2. Otherwise obtain an absolute home directory using `std::env::home_dir` and
   append `.local/state` or `.local/share`, respectively, on Linux; macOS uses
   `Library/Application Support` for both.
3. Return an error if neither provides a usable path.

Unset, empty, and relative XDG values all trigger the home fallback.
The Linux resolver logs unset variables at info level and non-absolute values (including
empty values) at warn level through `tracing`, explaining the fallback. The desktop
app enables these logs by default; `RUST_LOG` can override its filter. An absolute
XDG path does not require a home-directory lookup. Other operating systems
return an unsupported-platform error; the debug override is not native support
for those platforms.

## Verification

Linux and macOS tests inject environment values and a home-directory lookup into
resolver helpers. Follow that pattern instead of mutating process-wide
environment variables. Coverage includes XDG precedence, fallback behavior,
invalid homes, and non-Unicode paths for both state and share directories.
Public share-directory tests cover application names and the development override.
Log-directory tests cover all opinion values,
optional application names, and propagation of invalid-name errors.

Follow [Testing](../development/testing.md#platform-and-feature-variants), using
`-p platform-dirs` and checking the no-default-features configuration as well.
Linux resolver tests run only on Linux, so tests on another OS cannot verify
that resolver.

## Troubleshooting

- **XDG settings appear ignored:** default debug builds use the development
  override instead of native resolution.
- **A path resolves but writing fails:** path lookup does not create directories
  or check write permissions; those are the caller's responsibility.
- **Release startup fails outside Linux:** native state-directory resolution is
  currently unsupported there. The desktop treats initialization failure as
  fatal.
