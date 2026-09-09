# Repository guide

## Workflow

- See `CONTRIBUTING.md` for setup, checks, and PR expectations; visible desktop changes also require launching the app and exercising the affected interaction.
- Prefix commit subjects with `[desktop]` for desktop app changes and
  `[chess-core]` for chess core changes. Use `[desktop][chess-core]` when a
  commit changes both. Package-specific docs and tests follow the same rule;
  shared documentation and workspace-only changes require neither tag.

## Commands and toolchain

- Run Cargo commands from the workspace root with the Rust **1.98.0** toolchain pinned in `rust-toolchain.toml`.
- Launch the desktop GUI: `cargo run -p chessvault`.
- Checks: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo test --workspace`, `cargo clippy --workspace --all-targets`.
- Focus check/test/clippy on one package by replacing `--workspace` with `-p chessvault` or `-p chess-core`.
- Single GUI-package test: `cargo test -p chessvault --bin chessvault logs::tests::bridged_logs_follow_the_original_source_when_log_is_disabled -- --exact`.
- Single core test: `cargo test -p chess-core --lib tests::side_uses_one_byte -- --exact`.

## Package boundaries

- `apps/chessvault` is a binary-only GUI (entrypoint `src/main.rs`, tests in `src/logs.rs`); it does not yet depend on `crates/chess-core`.
- `crates/chess-core/src/lib.rs` contains private primitives and inline tests, not yet a public API. Tests pin `Side`/`Piece` to one-byte representations and their current discriminants.
- `Position` holds six piece-type and two side bitboards; `Piece::Empty` and `Side::Empty` are sentinels, never valid array indices.
- Use `Bitboard::empty()` for explicit zero-bit construction (`Default` delegates to it). `Position::empty()` zeroes all bitboards and sets `Side::Empty` to move; it is not the starting chess position, and `Position` has no `Default`.

## App wiring

- `apps/chessvault/src/main.rs` uses Iced **0.14**'s `iced::application` builder with `ChessVault::{update, view, subscription}`. Follow this API rather than older Iced `Application` trait examples.
- Iced's `tokio` feature enables the console's timer subscription. **F12** opens the developer console; its 250 ms refresh subscription runs only while open.
- The console uses a read-only `text_editor`: reject editing actions and preserve the selection during periodic refresh so copying remains usable.

## Logging invariants

- `Logs::init()` installs the global tracing subscriber before the GUI starts. Tests instead use scoped subscribers via `tracing::subscriber::with_default`.
- `RUST_LOG` controls capture (default `info`); use `RUST_LOG=chessvault=debug cargo run -p chessvault` for app debug events. Terminal output is enabled only with debug assertions, but in-app capture also works in release builds.
- Console source toggles filter snapshots, not capture; hidden events can be revealed later. `clear()` removes history but preserves source settings. “Copy all” copies the filtered snapshot.
- Sources come from the first `::`-separated component of the event target. Keep `NormalizeEvent` handling for bridged `log` records so dependency events retain their original source instead of becoming `log`.
- Publish each complete formatted event atomically on `LogWriter` drop; the shared buffer retains the latest 2,000 events, not lines.
