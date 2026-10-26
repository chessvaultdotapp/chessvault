# Repository guide

## Workflow

- Read `CONTRIBUTING.md` when preparing a PR; consult `docs/development/testing.md` for detailed verification. Visible
  desktop changes require launching the app in a graphical session and exercising the affected interaction.
- Commit subjects use adjacent tags: `[desktop]`, `[chess-core]`, `[platform-dirs]`, `[docs]`, `[ci]`.
  Tests retain package tags; shared docs use `[docs]`. See `docs/conventions/commit-messages.md` for combinations.
- Pin external GitHub Actions to full 40-character commit SHAs with release-version comments; verify both upstream when
  updating. Local actions use relative paths.

## Commands and toolchain

- Run Cargo commands from the workspace root using the toolchain pinned in `rust-toolchain.toml` and `--locked`.
- Launch the desktop GUI: `just run` or `cargo run -p chessvault --locked`; `just run-debug` enables app debug logs.
- `just check` runs build → nextest → doctests → formatting → Clippy, stopping on failure. Exact Cargo commands and
  setup are in `justfile` and `docs/development/testing.md` (also covers shell, justfile, and installer checks).
- Nextest excludes doctests. If unavailable, replace both test steps with `cargo test --workspace --locked`.
- Focus checks with `just check-package <package>` (`chessvault`, `chess-core`, `platform-dirs`).
  It keeps formatting workspace-wide and skips doctests for the binary-only desktop. Use workspace checks for shared
  configuration, cross-package changes, or APIs affecting consumers.
- Single desktop unit test (does not launch the GUI):
  `cargo test -p chessvault --bin chessvault --locked window_recreate_tests::saved_dimensions_round_trip -- --exact`.
- Single library test: `cargo test -p chess-core --lib --locked side::tests::side_uses_one_byte -- --exact`. For
  parameterized `rstest` tests, omit `--exact` to select all generated cases by function name.
- Markdown: `rumdl check <changed-files>` (120-column limit). TOML: `tombi format --check <changed-files>`, then
  `tombi lint --error-on-warnings <changed-files>`. CI checks all tracked files of each type; see the testing guide.
  Configuration/workflow-only edits can miss path-filtered CI; run relevant checks locally.

## Assets, documentation, and release tools

- The website lives at <https://github.com/chessvaultdotapp/website>; this repo publishes developer docs via
  `.github/workflows/docs.yml`.
- Workflow linting: `GOFLAGS=-mod=readonly go tool actionlint -shellcheck= -pyflakes= [workflow-files]` from the root;
  omit file arguments to check all workflows. CI checks only changed workflow YAML, not composite actions or installers.
- After `pnpm install --frozen-lockfile`, `just optimize-assets` optimizes app SVGs and regenerates diagrams.
  For diagrams only: `just --justfile crates/platform-dirs/justfile diagram`. Edit/commit `.mmd` sources;
  generated documentation SVGs are ignored, whereas application SVGs are tracked.
- `apps/chessvault/build.rs` renders `assets/logo.svg` into `OUT_DIR/window-icon.rgba` (embedded by `src/icon.rs`) and
  `OUT_DIR/chessvault.png` (distribution icon). Edit the SVG source; window pixels must use straight (demultiplied) RGBA.
- Build each Zensical root with `venvs/zensical/bin/zensical build --config-file <config>` after diagram generation:
  `zensical.toml`, `crates/chess-core/zensical.toml`, and `crates/platform-dirs/zensical.toml`.
  See `.github/workflows/docs.yml` for site assembly; use published URLs for cross-root links.
- Python setup: `just --justfile venvs/justfile development tag-tool dist-prepare` (requires uv). Follow
  `.github/workflows/python.yml`: Ruff/ty use `venvs/development/bin`, ty needs `--python venvs/tag-tool`, and pytest
  uses `venvs/tag-tool/bin/python` for both tag-tool and changelog-tool, `venvs/dist-prepare/bin/python` for dist-prepare.
- `tools/dist-prepare/main.py` only stages files: it requires `chessvault.png` beside the supplied binary and rejects an
  existing output directory. Building and archiving happen outside this tool.
- Installer tests: `just --justfile venvs/justfile install-script`, then
  `venvs/install-script/bin/python -m pytest scripts/tests -q` (Linux; no Rust build). Package smoke tests additionally
  need `CHESSVAULT_PACKAGE` pointing to an archive from the current checkout; see the testing guide.
- The tag tool requires a clean tree and cached Cargo dependencies, consumes committed `changelogs/unreleased.md` if present,
  and commits/tags locally. Pushing its development tag publishes a release; read `docs/tools/tag-tool.md` first.
- Before `just appimage`, follow `docs/development/appimage.md`; release packaging is in
  `.github/workflows/development-release.yml`. Zensical setup: `just --justfile venvs/justfile zensical`.

## Package boundaries

- The binary-only `apps/chessvault` depends directly on `crates/chess-core` and `crates/platform-dirs`.
- `chess-core` exposes types through `src/lib.rs` re-exports (including `Move`); bitboards and castling rights stay internal.
  Preserve tested one-byte representations/discriminants. `Piece::Empty` and `Side::Empty` are invalid occupancy indices;
  `Square::None` is an invalid bit index.
- Construct public positions with `Position::standard()`; `Position::empty()` is crate-private and there is no `Default`.
- `position/movegen.rs` owns legal move generation and application. `Position::play` rejects illegal moves without
  mutation; `Move` encodes castling with king squares and en passant with the pawn's landing square. Core generates
  all four promotions. Desktop must apply moves through `Game::play` to retain history and reject finished-game moves.
- `GameStatus::Checkmate(side)` and `Resigned(side)` identify the loser. Threefold/50-move draws require claims;
  fivefold/75-move draws are automatic. Intended-move claims do not play or record the move. Dead-position detection
  is conservative; blocked dead positions with pawns are not detected.
- Draw offers are made by the side to move and survive that player's move. Only the opponent can accept or decline;
  their next legal move implicitly declines. Invalid moves preserve the offer; any game result clears it.

## State paths

- `platform_dirs::{user_state_dir, user_share_dir, user_log_dir}` only resolve paths; callers create directories.
  Default-enabled `development` plus debug assertions selects
  `<cwd>/.local/state` and `<cwd>/.local/share` on every platform; release builds or `default-features = false` use OS
  resolution, currently Linux-only (other platforms error).
- Linux requires absolute XDG/home bases and preserves non-Unicode paths. Resolver tests inject environment/home
  values; do not mutate process-wide environment. Base resolution errors precede application-name validation.
- Path-policy changes also require `just check-platform-native` (the `platform-dirs` checks with
  `--no-default-features`); default debug checks do not exercise native resolution through the public entrypoint.
- Desktop state is normally `.local/state/chessvault/` in development; window dimensions use a 24-byte binary
  `recreate` file (no extension): a 16-byte versioned header and two little-endian `f32` values. Legacy JSON is rejected.
  `user_log_dir` appends `log` unless its opinion argument is `Some(false)`.

## App wiring

- `apps/chessvault/src/main.rs` uses Iced **0.14**'s `iced::application` builder with
  `ChessVault::{update, view, subscription}`. Follow this API rather than older Iced `Application` trait examples.
- `ChessVault` owns `Game`, `History`, and selection; `board.rs` calls core move APIs. Preserve embedded SVGs (`include_bytes!`)
  and cached `LazyLock` handles so rendering is directory-independent and reuses handles.
- History review reconstructs a separate position; never rewind or mutate the live game. Review is read-only;
  navigation is blocked while the console or promotion picker is open. Cache SAN labels when moves arrive, not on redraw.
- `pgn::export` emits headerless SAN movetext only for finished games, replaying from `Game::position_at(0)`;
  result tokens must use the losing-side semantics above.
- Promotion is deferred until the on-board picker chooses a piece. Outside clicks or Escape keep the pawn selected
  without changing the position; board moves are blocked while choosing.
- `.exit_on_close_request(false)` lets the close subscription query and save window size before `window::close`. Restore
  failures use defaults; save failures still close the window.
- Iced's `tokio` feature enables the **F12** console timer; refresh runs only while open. The read-only `text_editor`
  rejects editing actions and skips periodic refresh while text is selected so copying remains usable.
- `Logs::init()` installs the global tracing subscriber before the GUI starts. Tests instead use scoped subscribers via
  `tracing::subscriber::with_default`.
- `RUST_LOG` controls all logging (default `off,chessvault=info,platform_dirs=info`). Terminal output requires debug assertions;
  in-app capture works in release.
- Log-file initialization and background retention failures are nonfatal; subscriber installation failure is fatal.
  Console truncation notices must not promise an available log file. Retention (`logs/retention.rs`) preserves the
  current log, unrelated files, and symlinks; there is no cross-instance locking.
- Console source toggles filter snapshots, not capture; `clear()` preserves source settings and “Copy all” uses the
  filtered snapshot. Sources use the first `::`-separated target component; preserve `NormalizeEvent` for bridged `log`
  records. Publish complete events atomically through `Logs::store_event`; retention is 2,000 events, not lines.
