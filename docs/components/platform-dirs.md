# Platform directories

`platform-dirs` resolves the user's state-directory base. It does not know the
Chessvault application name and does not create directories. Application naming
belongs to [Application runtime](application-runtime.md).

## Source map

| File | Responsibility |
| --- | --- |
| [`lib.rs`](../../crates/platform-dirs/src/lib.rs) | Public API, development override, and platform selection |
| [`linux.rs`](../../crates/platform-dirs/src/linux.rs) | Linux resolution and injected-input tests |

## Public API

`platform_dirs::user_state_dir()` returns an `anyhow::Result<PathBuf>`. A
successful result identifies a path, not a guarantee that it exists or is
writable. Non-Unicode paths are preserved; display strings should not be used
to reconstruct paths.

### Development override

With both debug assertions and the default-enabled `development` feature, the
function returns:

```text
<current working directory>/.local/state
```

This works on every platform and does not consult `XDG_STATE_HOME`. An
unavailable working directory produces an error. The path is resolved on each
call, not cached, so changing the working directory changes later resolutions.

Consumers can disable default features on their dependency to use platform
resolution in debug builds. Cargo features are additive: another dependency
enabling `development` for the same package can re-enable the override.

### Native platform resolution

Release builds, and builds without `development`, use native resolution. Only
Linux is currently implemented:

1. Accept `XDG_STATE_HOME` when it is absolute.
2. Otherwise obtain an absolute home directory using `std::env::home_dir` and
   append `.local/state`.
3. Return an error if neither provides a usable path.

Unset, empty, and relative XDG values all trigger the home fallback. An absolute
XDG path does not require a home-directory lookup. Other operating systems
return an unsupported-platform error; the debug override is not native support
for those platforms.

## Verification

Linux tests inject environment values and a home-directory lookup into
`resolve_user_state_dir`. Follow that pattern instead of mutating process-wide
environment variables. Coverage includes XDG precedence, fallback behavior,
invalid homes, and non-Unicode paths.

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
