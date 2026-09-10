# Repository guide

## Workflow

- See `CONTRIBUTING.md` for setup, checks, and PR expectations; visible desktop changes also require launching the app and exercising the affected interaction.
- Prefix commit subjects with `[desktop]` for desktop app changes and
  `[chess-core]` for chess core changes. Use `[desktop][chess-core]` when a
  commit changes both. Package-specific docs and tests follow the same rule;
  shared documentation and workspace-only changes require neither package tag.
- Use `[ci]` for CI changes and CI-specific docs, combined with package tags when applicable.
- Pin external GitHub Actions to full 40-character commit SHAs with release-version comments; verify both upstream when updating. Local actions use relative paths.

## Commands and toolchain

- Run Cargo commands from the workspace root with the Rust **1.98.0** toolchain pinned in `rust-toolchain.toml`.
- Launch the desktop GUI: `cargo run -p chessvault`.
- Checks must run in order, stopping on failure: `cargo build --workspace --all-targets --locked` → `cargo test --workspace --locked` → `cargo fmt --all -- --check` → `cargo clippy --workspace --all-targets --locked --no-deps`.
- CI replaces the test step with `cargo nextest run --workspace --locked`, then `cargo test --workspace --doc --locked` (nextest excludes doctests). Local installation: `cargo install cargo-nextest --locked`; CI uses `.github/actions/setup-nextest`.
- Focus build/test/clippy by replacing `--workspace` with `-p chessvault`, `-p chess-core`, or `-p platform-dirs`; use workspace checks for shared configuration or cross-package changes.
- Single GUI-package test: `cargo test -p chessvault --bin chessvault --locked logs::tests::bridged_logs_follow_the_original_source_when_log_is_disabled -- --exact`.
- Single library test: `cargo test -p chess-core --lib --locked tests::side_uses_one_byte -- --exact`; platform-dirs tests use the `linux::tests::` prefix and run only on Linux.

## Package boundaries

- The workspace has three independent packages: `apps/chessvault`, `crates/chess-core`, and `crates/platform-dirs`. The binary-only GUI starts in `apps/chessvault/src/main.rs` and depends on neither library; its tests are in `src/logs.rs`.
- `apps/chessvault/src/board.rs` renders a responsive empty board, with no game state or core integration yet.
- `crates/chess-core/src/lib.rs` contains private primitives and inline tests, not yet a public API. Tests pin `Side`, `Piece`, and `CastlingRights` to one-byte representations and their current discriminants; castling rights are OR-combinable `u8` masks.
- `Position` holds six piece-type and two side bitboards; `Piece::Empty` and `Side::Empty` are sentinels, never valid array indices.
- Use `Bitboard::empty()` for explicit zero-bit construction (`Default` delegates to it). `Position::empty()` zeroes bitboards, castling rights, and both move counts, and sets `Side::Empty` to move; it is not the starting chess position, and `Position` has no `Default`.
- `platform-dirs::user_state_dir()` resolves a path without creating directories. Only Linux is implemented; other platforms return an error. Accept only absolute `XDG_STATE_HOME`, otherwise fall back to an absolute home directory plus `.local/state`; preserve non-Unicode paths.
- Platform-directory tests inject environment values and a home-directory lookup into `resolve_user_state_dir` in `crates/platform-dirs/src/linux.rs`; follow this pattern instead of mutating process-wide environment variables.

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
