# Contributing to Chessvault

## Development setup

Install Rust through [rustup](https://rustup.rs/). The repository pins Rust
1.98.0 in `rust-toolchain.toml`; rustup selects that toolchain automatically.
Run Cargo commands from the workspace root.

Launch the desktop app:

```sh
cargo run -p chessvault
```

## Workspace layout

- `apps/chessvault`: the desktop app, built with Iced 0.14.
- `crates/chess-core`: the chess core library and its inline unit tests.

The desktop app does not yet depend on `chess-core`.

## Commit messages

Prefix the commit subject with the tags for the packages or CI configuration it
changes:

- `[desktop]` for changes to the desktop app.
- `[chess-core]` for changes to the chess core.
- `[desktop][chess-core]` for a commit that changes both.
- `[ci]` for CI workflow changes and CI-specific documentation. Combine it with
  package tags when a commit also changes a package.

Use a short, imperative description after the tags:

```text
[desktop] Preserve console selection during refresh
[chess-core] Add empty position construction
[desktop][chess-core] Display core positions in the desktop app
[ci] Run tests before formatting and linting
```

Package-specific documentation and tests use the same package tag. Changes
limited to shared documentation or workspace infrastructure do not require
either package tag; use `[ci]` when those changes are CI-specific.

### Small, atomic, focused commits

Prefer small, atomic, focused commits. Each commit should represent one
complete, coherent change so it is easy to review, understand, and revert
independently. Keep the scope narrow without splitting a change into incomplete
or broken intermediate steps.

- Keep a feature or bug fix together with its relevant tests and documentation.
- Put unrelated refactoring, formatting, or cleanup in separate commits.
- Keep cross-package changes together when they serve the same purpose, using
  both `[desktop][chess-core]` tags.
- Aim for each commit to build and pass the relevant checks on its own.

For example, a desktop console fix and its regression test belong in one
`[desktop]` commit. An unrelated chess-core refactor belongs in a separate
`[chess-core]` commit.

## Checks

Before submitting changes, build first, then run tests, formatting, and linting
in that order. CI follows the same rule: a failed build or test run skips
formatting and linting.

CI uses `cargo nextest run --workspace --locked` for tests, followed by
`cargo test --workspace --doc --locked` because nextest does not run doctests.
To use the same runner locally, install it with
`cargo install cargo-nextest --locked`.

The local action at `.github/actions/setup-nextest` installs official prebuilt
binaries directly from `get.nexte.st`. It supports x64 and ARM64 runners on
Linux, Windows, and macOS, using Bash (Git Bash on Windows). After checkout,
workflows can use it with:

```yaml
- uses: ./.github/actions/setup-nextest
  with:
    version: latest
```

The optional `version` input defaults to `latest` and also accepts a release
series such as `0.9` or an exact release version.

```sh
cargo build --workspace --all-targets --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
```

For package-local changes, replace `--workspace` with `-p chessvault` or
`-p chess-core`. Use workspace-wide checks for changes affecting both packages
or shared configuration.

For visible desktop changes, also launch the app and exercise the affected
interaction. Include a brief description of what changed and how it was
verified in the pull request; screenshots are useful for UI changes.

## GitHub Actions

Pin all external GitHub Actions used in workflows and composite actions to a
full 40-character commit SHA, rather than a mutable tag or branch. Include the
corresponding release version in an inline comment for readability:

```yaml
- uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
```

When updating an action, verify that the SHA belongs to the intended release
in the action's upstream repository, and update both the SHA and version comment.
Repository-local actions use relative paths, such as
`uses: ./.github/actions/setup-nextest`; their version comes from the checked-out
repository commit.

## Desktop debugging

Press **F12** to open the developer console. Enable app debug events with:

```sh
RUST_LOG=chessvault=debug cargo run -p chessvault
```

`RUST_LOG` controls which events are captured. The console's source toggles
filter captured events, so hidden sources can be shown again later.

See [AGENTS.md](AGENTS.md) for detailed implementation conventions and
invariants.
