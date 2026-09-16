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
- Focus build/test/clippy by replacing `--workspace` with `-p chessvault`, `-p chess-core`, `-p application-runtime`, or `-p platform-dirs`; use workspace checks for shared configuration or cross-package changes.
- Single GUI test: `cargo test -p chessvault --bin chessvault --locked window_recreate_tests::saved_dimensions_round_trip -- --exact`.
- Single library test: `cargo test -p chess-core --lib --locked side::tests::side_uses_one_byte -- --exact`. For parameterized `rstest` tests, omit `--exact` to select all generated cases by function name.

## Package boundaries

- Four packages: the binary-only `apps/chessvault` depends on `crates/chess-core` and `crates/application-runtime`; the runtime depends on `crates/platform-dirs`. The workspace layout in `CONTRIBUTING.md` is incomplete; manifests are authoritative.
- `crates/chess-core/src/lib.rs` keeps modules private and re-exports `Piece`, `Position`, `Side`, `Square`, and `InvalidSquareIndex`; bitboards and castling rights remain internal. Tests live beside implementations.
- Tests pin `Square`, `Side`, `Piece`, and `CastlingRights` to one-byte representations and their discriminants; castling rights are OR-combinable `u8` masks.
- `Position` holds six piece-type and two side bitboards; `Piece::Empty` and `Side::Empty` are never valid array indices. Squares are rank-major (`A1 = 0`, `H8 = 63`); `Square::None` is not a valid bit index.
- Use `Bitboard::empty()` for zero-bit construction (`Default` delegates to it). Crate-private `Position::empty()` zeroes boards, rights, and counts, with `Side::Empty` to move; public `Position::standard()` sets up the starting position. `Position` has no `Default`.

## State paths

- `platform_dirs::user_state_dir()` only resolves paths. Default-enabled `development` plus debug assertions selects `<cwd>/.local/state` on every platform; release builds or `default-features = false` use OS resolution, currently Linux-only (other platforms error).
- Linux accepts only absolute `XDG_STATE_HOME`, falling back to an absolute home directory plus `.local/state`. Preserve non-Unicode paths; resolver tests inject environment values and home lookup instead of mutating process-wide environment. Linux tests use `linux::tests::` and are Linux-only.
- `application_runtime::fs::application_state_dir()` appends `chessvault`; desktop initialization creates the directory. Window dimensions are JSON in `recreate` (no extension), normally `.local/state/chessvault/recreate` in development.

## App wiring

- `apps/chessvault/src/main.rs` uses Iced **0.14**'s `iced::application` builder with `ChessVault::{update, view, subscription}`. Follow this API rather than older Iced `Application` trait examples.
- `ChessVault` owns a `Position::standard()`; `board.rs` renders via `Position::piece_at`. Piece SVGs are embedded with `include_bytes!` and cached in `LazyLock` handles; preserve working-directory-independent rendering and handle reuse.
- `.exit_on_close_request(false)` lets the close subscription query and save window size before `window::close`. Restore failures use defaults; save failures still close the window.
- Iced's `tokio` feature enables the console's timer subscription. **F12** opens the developer console; its 250 ms refresh subscription runs only while open.
- The console uses a read-only `text_editor`: reject editing actions and preserve the selection during periodic refresh so copying remains usable.

## Logging invariants

- `Logs::init()` installs the global tracing subscriber before the GUI starts. Tests instead use scoped subscribers via `tracing::subscriber::with_default`.
- `RUST_LOG` controls capture (default `info`); use `RUST_LOG=chessvault=debug cargo run -p chessvault` for app debug events. Terminal output is enabled only with debug assertions, but in-app capture also works in release builds.
- Console source toggles filter snapshots, not capture; hidden events can be revealed later. `clear()` removes history but preserves source settings. “Copy all” copies the filtered snapshot.
- Sources come from the first `::`-separated component of the event target. Keep `NormalizeEvent` handling for bridged `log` records so dependency events retain their original source instead of becoming `log`.
- Publish each complete formatted event atomically on `LogWriter` drop; the shared buffer retains the latest 2,000 events, not lines.
