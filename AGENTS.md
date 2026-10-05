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

- Run Cargo commands from the workspace root using the toolchain pinned in `rust-toolchain.toml` and `--locked`.
- Launch the desktop GUI: `just run` or `cargo run -p chessvault --locked`; `just run-debug` enables app debug logs.
- `just check` runs build → nextest → doctests → formatting → Clippy, stopping on failure. Exact Cargo commands and
  setup are in `justfile` and `docs/development/testing.md`; Clippy uses `--no-deps`.
- Nextest excludes doctests. If unavailable, replace both test steps with `cargo test --workspace --locked`.
- Focus checks with `just check-package <package>` (`chessvault`, `chess-core`, `application-runtime`, `platform-dirs`).
  It keeps formatting workspace-wide and skips doctests for the binary-only desktop. Use workspace checks for shared
  configuration, cross-package changes, or APIs affecting consumers.
- Single desktop unit test (does not launch the GUI):
  `cargo test -p chessvault --bin chessvault --locked window_recreate_tests::saved_dimensions_round_trip -- --exact`.
- Single library test: `cargo test -p chess-core --lib --locked side::tests::side_uses_one_byte -- --exact`. For
  parameterized `rstest` tests, omit `--exact` to select all generated cases by function name.
- Markdown: `rumdl check <changed-files>` (120-column limit). TOML: `tombi format --check <changed-files>`, then
  `tombi lint --error-on-warnings <changed-files>`. CI checks all tracked files of each type; see the testing guide.
  Workflow-only edits do not trigger Markdown/TOML checks or Rust CI; run relevant checks locally.

## Assets, documentation, and release tools

- Workflow linting: `GOFLAGS=-mod=readonly go tool actionlint -shellcheck= -pyflakes= [workflow-files]` from the root;
  omit file arguments to check all workflows. CI checks only changed workflow YAML, not composite actions or installers.
- `pnpm install --frozen-lockfile` installs SVG tooling. `just optimize-assets` optimizes desktop SVGs and regenerates
  documentation diagrams; it requires Graphviz. Edit diagram `.dot` sources, then run
  `just --justfile crates/platform-dirs/justfile diagram` for a diagram-only update.
- `apps/chessvault/build.rs` renders `assets/logo.svg` into `OUT_DIR/window-icon.rgba` (embedded by `src/icon.rs`) and
  `OUT_DIR/chessvault.png` (distribution icon). Edit the SVG source; window pixels must use straight (demultiplied) RGBA.
- Documentation has two Zensical roots. Build both with `venvs/zensical/bin/zensical build --config-file zensical.toml`
  and `venvs/zensical/bin/zensical build --config-file crates/platform-dirs/zensical.toml` after diagram generation.
  Cross-root links are not yet staged; see `.github/workflows/docs.yml` for build/deployment setup.
- With uv installed: `just --justfile venvs/justfile development tag-tool dist-prepare zensical` creates Python environments.
  Python checks use `venvs/development/bin/ruff check`, `venvs/development/bin/ruff format --check`, and
  `venvs/development/bin/ty check --python venvs/tag-tool`, passing changed Python paths to each.
  Test with `venvs/tag-tool/bin/python -m pytest tools/tag-tool -q` and
  `venvs/dist-prepare/bin/python -m pytest tools/dist-prepare -q`; `.github/workflows/python.yml` is the source of truth.
- `tools/dist-prepare/main.py` only stages files: it requires `chessvault.png` beside the supplied binary and rejects an
  existing output directory. Building and archiving happen outside this tool.
- `tools/tag-tool/main.py` requires a clean tree and cached Cargo dependencies: it updates the desktop version and
  lockfile offline, consumes committed `changelogs/unreleased.md` if present, then commits and tags locally.
  Pushing a development tag triggers publication; see `docs/tools/tag-tool.md` for release and recovery instructions.
- `just appimage` builds the Linux x86_64 AppImage; consult `docs/development/appimage.md` for prerequisites and
  cross-distribution acceptance checks, and `.github/workflows/development-release.yml` for release packaging.

## Package boundaries

- The binary-only `apps/chessvault` depends on `crates/chess-core` and `crates/application-runtime`; only the runtime
  depends on `crates/platform-dirs`.
- `chess-core` exposes types through `src/lib.rs` re-exports (including `Move`); bitboards and castling rights stay internal.
- Tests pin one-byte representations and discriminants for squares, sides, pieces, and castling-right masks.
  `Piece::Empty` and `Side::Empty` are invalid occupancy indices; `Square::None` is an invalid bit index.
- Construct public positions with `Position::standard()`; `Position::empty()` is crate-private and there is no `Default`.
- `position/movegen.rs` owns legal move generation and application. `Position::play` rejects illegal moves without
  mutation; `Move` encodes castling with king squares and en passant with the pawn's landing square. Core generates
  all four promotions; desktop `board::click` currently chooses a queen.

## State paths

- `platform_dirs::user_state_dir()` only resolves paths. Default-enabled `development` plus debug assertions selects
  `<cwd>/.local/state` on every platform; release builds or `default-features = false` use OS resolution, currently
  Linux-only (other platforms error).
- Linux accepts only absolute `XDG_STATE_HOME`, falling back to an absolute home directory plus `.local/state`. Preserve
  non-Unicode paths; resolver tests inject environment values and home lookup instead of mutating process-wide
  environment. Linux resolver tests are Linux-only.
- Path-policy changes also require `just check-platform-native` (the `platform-dirs` checks with
  `--no-default-features`); default debug checks do not exercise native resolution through the public entrypoint.
- `application_runtime::fs::application_state_dir()` appends `chessvault`; desktop initialization creates the directory.
  Window dimensions are JSON in `recreate` (no extension), normally `.local/state/chessvault/recreate` in development.

## App wiring

- `apps/chessvault/src/main.rs` uses Iced **0.14**'s `iced::application` builder with
  `ChessVault::{update, view, subscription}`. Follow this API rather than older Iced `Application` trait examples.
- `ChessVault` owns position/selection; `board.rs` calls core move APIs. Preserve embedded piece SVGs (`include_bytes!`)
  and cached `LazyLock` handles so rendering is directory-independent and reuses handles.
- `.exit_on_close_request(false)` lets the close subscription query and save window size before `window::close`. Restore
  failures use defaults; save failures still close the window.
- Iced's `tokio` feature enables the console timer. **F12** opens the console; refresh runs only while open.
- The console uses a read-only `text_editor`: reject editing actions and preserve the selection during periodic refresh
  so copying remains usable.
- `Logs::init()` installs the global tracing subscriber before the GUI starts. Tests instead use scoped subscribers via
  `tracing::subscriber::with_default`.
- `RUST_LOG` controls capture (default `info`). Terminal output requires debug assertions; in-app capture works in release.
- Console source toggles filter snapshots, not capture; hidden events can be revealed later. `clear()` removes history
  but preserves source settings. “Copy all” copies the filtered snapshot.
- Sources come from the first `::`-separated component of the event target. Keep `NormalizeEvent` handling for bridged
  `log` records so dependency events retain their original source instead of becoming `log`.
- Publish each complete formatted event atomically on `LogWriter` drop; the shared buffer retains the latest 2,000
  events, not lines.
