# Repository guide

## Workflow

- Read `CONTRIBUTING.md` when preparing a PR; consult `docs/development/testing.md` for detailed verification. Visible
  desktop changes require launching the app in a graphical session and exercising the affected interaction.
- Commit tags follow `docs/conventions/commit-messages.md`: `[desktop]`, `[chess-core]`, or
  `[platform-dirs]` for the affected packages (including tests); add `[docs]` for documentation and `[ci]` for CI.
  Combine applicable tags without spaces. Shared docs use `[docs]`; workspace-only infrastructure needs no package tag.
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

## Website

- `website/` is a separate Astro pnpm workspace, not covered by `just check`; read `website/AGENTS.md` when working there.
- From the root: `pnpm --filter website install --frozen-lockfile`, then
  `pnpm --filter website exec oxfmt --check src astro.config.mjs`,
  `pnpm --filter website exec oxlint --deny-warnings src astro.config.mjs`, and
  `pnpm --filter website exec astro check`, stopping on failure.
- Also run `pnpm --filter website build`; preview and exercise visible changes in a browser. Deployment uses a non-root
  base path; `.github/workflows/docs.yml` builds with `--site https://chessvaultdotapp.github.io --base /chessvault/`.

## Assets, documentation, and release tools

- Workflow linting: `GOFLAGS=-mod=readonly go tool actionlint -shellcheck= -pyflakes= [workflow-files]` from the root;
  omit file arguments to check all workflows. CI checks only changed workflow YAML, not composite actions or installers.
- `pnpm install --frozen-lockfile` installs asset tooling. `just optimize-assets` also regenerates diagrams. Edit `.mmd`
  sources, then run
  `just --justfile crates/platform-dirs/justfile diagram` for a diagram-only update. The two generated documentation
  SVGs are ignored by Git; commit `.mmd` sources, not generated diagrams. Application SVG assets remain tracked.
- `apps/chessvault/build.rs` renders `assets/logo.svg` into `OUT_DIR/window-icon.rgba` (embedded by `src/icon.rs`) and
  `OUT_DIR/chessvault.png` (distribution icon). Edit the SVG source; window pixels must use straight (demultiplied) RGBA.
- Documentation has two Zensical roots: run `venvs/zensical/bin/zensical build --config-file <config>` for both
  `zensical.toml` and `crates/platform-dirs/zensical.toml` after diagram generation. See `.github/workflows/docs.yml`
  for site assembly; cross-root links are not yet staged.
- With uv installed: `just --justfile venvs/justfile development tag-tool dist-prepare zensical` creates Python environments.
  Python checks use `venvs/development/bin/ruff check`, `venvs/development/bin/ruff format --check`, and
  `venvs/development/bin/ty check --python venvs/tag-tool`, passing changed Python paths to each.
  Test with `venvs/tag-tool/bin/python -m pytest tools/tag-tool -q` and
  `venvs/dist-prepare/bin/python -m pytest tools/dist-prepare -q`; `.github/workflows/python.yml` is the source of truth.
- `tools/dist-prepare/main.py` only stages files: it requires `chessvault.png` beside the supplied binary and rejects an
  existing output directory. Building and archiving happen outside this tool.
- Installer tests: `just --justfile venvs/justfile install-script`, then
  `venvs/install-script/bin/python -m pytest scripts/tests -q` (Linux; no Rust build). Package smoke tests additionally
  need `CHESSVAULT_PACKAGE` pointing to an archive from the current checkout; see the testing guide.
- `tools/tag-tool/main.py` requires a clean tree and cached Cargo dependencies: it updates the desktop version and
  lockfile offline, consumes committed `changelogs/unreleased.md` if present, then commits and tags locally.
  Pushing a development tag triggers publication; see `docs/tools/tag-tool.md` for release and recovery instructions.
- `just appimage` requires the setup in `docs/development/appimage.md`; release packaging lives in
  `.github/workflows/development-release.yml`.

## Package boundaries

- The binary-only `apps/chessvault` depends directly on `crates/chess-core` and `crates/platform-dirs`.
- `chess-core` exposes types through `src/lib.rs` re-exports (including `Move`); bitboards and castling rights stay internal.
- Tests pin one-byte representations and discriminants for squares, sides, pieces, and castling-right masks.
  `Piece::Empty` and `Side::Empty` are invalid occupancy indices; `Square::None` is an invalid bit index.
- Construct public positions with `Position::standard()`; `Position::empty()` is crate-private and there is no `Default`.
- `position/movegen.rs` owns legal move generation and application. `Position::play` rejects illegal moves without
  mutation; `Move` encodes castling with king squares and en passant with the pawn's landing square. Core generates
  all four promotions. Desktop must apply moves through `Game::play` to retain history and reject finished-game moves.
- `GameStatus::Checkmate(side)` identifies the loser, not the winner. Draw currently means stalemate only.

## State paths

- `platform_dirs::{user_state_dir, user_log_dir}` only resolve paths; callers create directories.
  Default-enabled `development` plus debug assertions selects
  `<cwd>/.local/state` on every platform; release builds or `default-features = false` use OS resolution, currently
  Linux-only (other platforms error).
- Linux accepts only absolute `XDG_STATE_HOME`, falling back to an absolute home directory plus `.local/state`. Preserve
  non-Unicode paths; resolver tests inject environment values and home lookup instead of mutating process-wide
  environment. Linux resolver tests are Linux-only.
- Path-policy changes also require `just check-platform-native` (the `platform-dirs` checks with
  `--no-default-features`); default debug checks do not exercise native resolution through the public entrypoint.
- Desktop uses `Some("chessvault")`; `None` returns the base. App names must be one nonempty component, excluding
  `.`, `..`, slashes, backslashes, colons, and NUL. Window dimensions are JSON in `recreate` (no extension), normally
  `.local/state/chessvault/recreate` in development.
  `user_log_dir` appends `log` unless its opinion argument is `Some(false)`.

## App wiring

- `apps/chessvault/src/main.rs` uses Iced **0.14**'s `iced::application` builder with
  `ChessVault::{update, view, subscription}`. Follow this API rather than older Iced `Application` trait examples.
- `ChessVault` owns `Game`, `History`, and selection; `board.rs` calls core move APIs. Preserve embedded SVGs (`include_bytes!`)
  and cached `LazyLock` handles so rendering is directory-independent and reuses handles.
- History review reconstructs a separate position; never rewind or mutate the live game. Review is read-only;
  navigation is blocked while the console or promotion picker is open. Cache SAN labels when moves arrive, not on redraw.
- Promotion is deferred until the on-board picker chooses a piece. Outside clicks or Escape keep the pawn selected
  without changing the position; board moves are blocked while choosing.
- `.exit_on_close_request(false)` lets the close subscription query and save window size before `window::close`. Restore
  failures use defaults; save failures still close the window.
- Iced's `tokio` feature enables the **F12** console timer; refresh runs only while open.
  Its read-only `text_editor` rejects editing actions and preserves the selection during periodic refresh
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
