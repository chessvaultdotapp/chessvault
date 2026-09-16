# Testing

Commands follow our [shell command conventions](../reference.md#shell-commands).
Run them from the workspace root using the toolchain selected by
[`rust-toolchain.toml`](../../rust-toolchain.toml). See
[Getting started](getting-started.md#install-cargo-nextest) for runner installation.

## Ordered checks

Run these checks in order, stopping if a command fails. Fix the failure before
continuing:

```console
$ cargo build --workspace --all-targets --locked
$ cargo nextest run --workspace --locked
$ cargo test --workspace --doc --locked
$ cargo fmt --all -- --check
$ cargo clippy --workspace --all-targets --locked --no-deps
```

This is the sequence used by [CI](../../.github/workflows/ci.yml). Nextest is our
preferred test runner, but it does not execute doctests, so the separate
`cargo test --doc` step is required. Formatting checks do not modify files.
Clippy's `--no-deps` excludes dependency linting, though dependencies may still
need to be compiled or checked.

If nextest is unavailable, replace both test commands with
`cargo test --workspace --locked`, which runs unit tests, integration tests, and
doctests. Keep the build, formatting, and Clippy steps in the same order.

## Focused checks

For package-local changes, replace `--workspace` in the build, test, and Clippy
commands with one of:

- `-p chessvault` — desktop application.
- `-p chess-core` — chess primitives and positions.
- `-p application-runtime` — application state-directory policy.
- `-p platform-dirs` — platform directory resolution.

Keep `cargo fmt --all -- --check` workspace-wide. Use workspace checks for shared
configuration, cross-package changes, and API changes affecting consumers.

For example, the core-only sequence is:

```console
$ cargo build -p chess-core --all-targets --locked
$ cargo nextest run -p chess-core --locked
$ cargo test -p chess-core --doc --locked
$ cargo fmt --all -- --check
$ cargo clippy -p chess-core --all-targets --locked --no-deps
```

### A single test

After a successful build, use nextest's filter expression to select a test by
its exact name:

```console
$ cargo nextest run -p chess-core --locked -E 'test(=side::tests::side_uses_one_byte)'
```

The equivalent Cargo test command explicitly selects the library target:

```console
$ cargo test -p chess-core --lib --locked side::tests::side_uses_one_byte -- --exact
```

The desktop is binary-only. For example, to test saved window dimensions:

```console
$ cargo test -p chessvault --bin chessvault --locked window_recreate_tests::saved_dimensions_round_trip -- --exact
```

Focused tests shorten iteration; they do not replace the relevant full check
sequence before submitting changes. See the
[nextest filtering documentation](https://nexte.st/docs/filtersets/) for more
selection options.

## Test organization and isolation

Tests currently live in inline `#[cfg(test)]` modules beside implementations.
Use these nearby tests as examples when extending a component:

- **Chess core:** pin compact representations and discriminants, bitboard
  operations, position construction, and square lookup. Include sentinel cases;
  empty pieces, sides, and squares are not valid occupancy indices.
- **Desktop:** test log capture and filtering, window-state JSON validation, and
  file-path composition independently of the GUI.
- **Runtime and platform directories:** inject upstream path results,
  environment values, and home-directory lookups. Do not mutate process-wide
  environment variables in tests. Preserve non-Unicode path coverage where
  supported.
- **Logging:** use scoped subscribers via `tracing::subscriber::with_default`,
  not `Logs::init()`, which installs the application's global subscriber.

Put executable public API examples in Rustdoc when they should be covered by
doctests. Code blocks in Markdown guides are not automatically tested by Cargo.

## Platform and feature variants

The default-enabled `platform-dirs` development feature makes debug builds use
working-directory-local state paths. Native resolution takes a different
compiled branch, so default debug checks alone do not cover every configuration.

For changes to that policy, also run:

```console
$ cargo build -p platform-dirs --all-targets --locked --no-default-features
$ cargo nextest run -p platform-dirs --locked --no-default-features
$ cargo test -p platform-dirs --doc --locked --no-default-features
$ cargo fmt --all -- --check
$ cargo clippy -p platform-dirs --all-targets --locked --no-deps --no-default-features
```

Stop on failure here too. Linux resolver tests are compiled only on Linux and
use the `linux::tests::` prefix. Passing tests on another OS does not verify
Linux resolution. Native state-directory resolution currently supports only
Linux; the development path working elsewhere is not evidence of release app
support. See [Runtime and storage](../runtime-and-storage.md) for the path policy.

## Desktop interaction checks

Automated tests do not replace launching the app for visible desktop changes:

```console
$ cargo run -p chessvault --locked
```

Exercise the affected interaction in a graphical desktop session. Depending on
the change, verify board resizing and orientation, F12 console toggling, source
filtering, copying selected logs during refresh, or saving and restoring window
size. See [Desktop application](../desktop.md#verification) for more guidance.

Include a brief description of what changed and how it was verified in the pull
request. Screenshots are useful for UI changes. Report any checks you could not
run and why, rather than treating them as passed.
