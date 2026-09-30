# Repository guide

## Workflow

- Read `CONTRIBUTING.md` when preparing a PR; consult `docs/development/testing.md` for detailed verification. Visible
  desktop changes require launching the app in a graphical session and exercising the affected interaction.
- Commit tags follow `docs/conventions/commit-messages.md`: `[desktop]`, `[chess-core]`, `[application-runtime]`, or
  `[platform-dirs]` for the affected packages (including tests); add `[docs]` for documentation and `[ci]` for CI.
  Combine applicable tags without spaces. Shared docs use `[docs]`; workspace-only infrastructure needs no package tag.
- Pin external GitHub Actions to full 40-character commit SHAs with release-version comments; verify both upstream when
  updating. Local actions use relative paths.

## Commands and toolchain

- Run Cargo commands from the workspace root with the Rust **1.98.0** toolchain pinned in `rust-toolchain.toml`.
- Launch the desktop GUI: `just run` or `cargo run -p chessvault --locked`; `just run-debug` enables app debug logs.
- `just check` runs the local/CI sequence below, stopping on failure:

  ```sh
  cargo build --workspace --all-targets --locked
  cargo nextest run --workspace --locked
  cargo test --workspace --doc --locked
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --locked --no-deps
  ```

- Nextest excludes doctests. Install with `cargo install cargo-nextest --locked`; if unavailable, replace both test
  steps with `cargo test --workspace --locked`. CI installs via `.github/actions/setup-nextest`.
- Focus checks with `just check-package <package>` (`chessvault`, `chess-core`, `application-runtime`, `platform-dirs`).
  It keeps formatting workspace-wide and skips doctests for the binary-only desktop. Use workspace checks for shared
  configuration, cross-package changes, or APIs affecting consumers.
- Single GUI test:
  `cargo test -p chessvault --bin chessvault --locked window_recreate_tests::saved_dimensions_round_trip -- --exact`.
- Single library test: `cargo test -p chess-core --lib --locked side::tests::side_uses_one_byte -- --exact`. For
  parameterized `rstest` tests, omit `--exact` to select all generated cases by function name.
- Markdown: `rumdl check <changed-files>` (120-column limit). TOML: `tombi format --check <changed-files>`, then
  `tombi lint --error-on-warnings <changed-files>`. CI checks all tracked files of each type; see the testing guide.
  Workflow-only edits do not trigger these workflows or Rust CI; run relevant checks locally.
- SVG optimization: install dependencies with `pnpm install --frozen-lockfile` in `apps/chessvault`, then run
  `just optimize-assets` from the root. Node/pnpm tooling is for artwork, not the desktop runtime.

## Package boundaries

- Four packages: the binary-only `apps/chessvault` depends on `crates/chess-core` and `crates/application-runtime`; the
  runtime depends on `crates/platform-dirs`. `CONTRIBUTING.md`'s workspace section is stale (including its claim that
  desktop does not depend on core); trust manifests.
- `crates/chess-core/src/lib.rs` exposes types through re-exports (including `Move`); bitboards and castling rights
  remain internal. Tests live beside implementations.
- Tests pin `Square`, `Side`, `Piece`, and `CastlingRights` to one-byte representations and their discriminants;
  castling rights are OR-combinable `u8` masks.
- `Position` holds six piece-type and two side bitboards; `Piece::Empty` and `Side::Empty` are never valid array
  indices. Squares are rank-major (`A1 = 0`, `H8 = 63`); `Square::None` is not a valid bit index.
- `Position::standard()` is the public constructor; `Position::empty()` is crate-private and there is no `Default`.
- `position/movegen.rs` owns legal move generation and application. `Position::play` rejects illegal moves without
  mutation; `Move` encodes castling with king squares and en passant with the pawn's landing square. Core generates
  all four promotions; desktop `board::click` currently chooses a queen.

## State paths

- `platform_dirs::user_state_dir()` only resolves paths. Default-enabled `development` plus debug assertions selects
  `<cwd>/.local/state` on every platform; release builds or `default-features = false` use OS resolution, currently
  Linux-only (other platforms error).
- Linux accepts only absolute `XDG_STATE_HOME`, falling back to an absolute home directory plus `.local/state`. Preserve
  non-Unicode paths; resolver tests inject environment values and home lookup instead of mutating process-wide
  environment. Linux tests use `linux::tests::` and are Linux-only.
- Path-policy changes also require `just check-platform-native` (the `platform-dirs` checks with
  `--no-default-features`); default debug checks do not exercise native resolution through the public entrypoint.
- `application_runtime::fs::application_state_dir()` appends `chessvault`; desktop initialization creates the directory.
  Window dimensions are JSON in `recreate` (no extension), normally `.local/state/chessvault/recreate` in development.

## App wiring

- `apps/chessvault/src/main.rs` uses Iced **0.14**'s `iced::application` builder with
  `ChessVault::{update, view, subscription}`. Follow this API rather than older Iced `Application` trait examples.
- `ChessVault` owns the position and selection; `board.rs` handles clicks and legal-destination hints via core APIs.
  Piece SVGs use `include_bytes!` and cached `LazyLock` handles; preserve directory-independent rendering and handle reuse.
- `.exit_on_close_request(false)` lets the close subscription query and save window size before `window::close`. Restore
  failures use defaults; save failures still close the window.
- Iced's `tokio` feature enables the console's timer subscription. **F12** opens the developer console; its 250 ms
  refresh subscription runs only while open.
- The console uses a read-only `text_editor`: reject editing actions and preserve the selection during periodic refresh
  so copying remains usable.

## Logging invariants

- `Logs::init()` installs the global tracing subscriber before the GUI starts. Tests instead use scoped subscribers via
  `tracing::subscriber::with_default`.
- `RUST_LOG` controls capture (default `info`); use `RUST_LOG=chessvault=debug cargo run -p chessvault --locked` for app
  debug events. Terminal output is enabled only with debug assertions, but in-app capture also works in release builds.
- Console source toggles filter snapshots, not capture; hidden events can be revealed later. `clear()` removes history
  but preserves source settings. “Copy all” copies the filtered snapshot.
- Sources come from the first `::`-separated component of the event target. Keep `NormalizeEvent` handling for bridged
  `log` records so dependency events retain their original source instead of becoming `log`.
- Publish each complete formatted event atomically on `LogWriter` drop; the shared buffer retains the latest 2,000
  events, not lines.
