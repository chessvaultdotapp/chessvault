# Platform directories

`platform-dirs` resolves the base directory for persistent user state. It only
resolves paths: callers append application-specific names and create directories
as needed.

## API

The crate exposes `platform_dirs::user_state_dir()`, which returns an
`anyhow::Result<PathBuf>`. See the [in-code documentation](../src/lib.rs) for
resolution rules, errors, and usage examples.

## Features and platform support

With debug assertions and the default-enabled `development` feature, resolution
uses `<current working directory>/.local/state` on every platform.

Otherwise, native resolution is used. Currently, only Linux is supported:
an absolute `XDG_STATE_HOME` takes precedence, with a fallback to `.local/state`
beneath an absolute home directory. Other platforms return an error.

Resolution preserves non-Unicode paths and does not check whether the directory
exists or is writable.

## Further reading

- [Architecture](architecture.md): boundaries, module structure, resolution flow,
  and testing strategy
- [Workspace component guide](../../../docs/components/platform-dirs.md)
- [Testing and feature variants](../../../docs/development/testing.md#platform-and-feature-variants)
