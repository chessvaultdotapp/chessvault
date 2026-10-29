# Repository guide

## Workflow

- Read `CONTRIBUTING.md` when preparing a PR; consult `docs/development/testing.md` for detailed verification.
- Desktop-specific guidance lives in `apps/chessvault/AGENTS.md`.
- Commit subjects use adjacent tags: `[desktop]`, `[chess-core]`, `[platform-dirs]`, `[docs]`, `[ci]`.
  Tests retain package tags; shared docs use `[docs]`. See `docs/conventions/commit-messages.md` for combinations.
- Pin external GitHub Actions to full 40-character commit SHAs with release-version comments; verify both upstream when
  updating. Local actions use relative paths.

## Commands and toolchain

- Run Cargo commands from the workspace root using the toolchain pinned in `rust-toolchain.toml` and `--locked`.
- `just check` runs build → nextest → doctests → formatting → Clippy, stopping on failure. Exact Cargo commands and
  setup are in `justfile` and `docs/development/testing.md` (also covers shell, justfile, and installer checks).
- Nextest excludes doctests. If unavailable, replace both test steps with `cargo test --workspace --locked`.
- Focus checks with `just check-package <package>` (`chessvault`, `chess-core`, `platform-dirs`).
  It keeps formatting workspace-wide and skips doctests for the binary-only desktop. Use workspace checks for shared
  configuration, cross-package changes, or APIs affecting consumers.
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
- Documentation has four independent Zensical roots; build all as in `.github/workflows/docs.yml` after diagram generation.
  Use published URLs for cross-root links. Setup: `just --justfile venvs/justfile zensical`.
- Python setup requires uv; follow `.github/workflows/python.yml` for environment-specific checks rather than using
  system Python. Ruff/ty live in `venvs/development/bin`; tag-tool and changelog-tool tests use `venvs/tag-tool/bin/python`,
  dist-prepare tests use `venvs/dist-prepare/bin/python`. Differential tests need their own environment, including for ty.
- Installer tests: `just --justfile venvs/justfile install-script`, then
  `venvs/install-script/bin/python -m pytest scripts/tests -q` (Linux; no Rust build). Package smoke tests additionally
  need `CHESSVAULT_PACKAGE` pointing to an archive from the current checkout; see the testing guide.
- The tag tool requires a clean tree and cached Cargo dependencies, consumes committed `changelogs/unreleased.md` if present,
  and commits/tags locally. Pushing its development tag publishes a release; read `docs/tools/tag-tool.md` first.
- Before packaging, read `docs/development/appimage.md` and `.github/workflows/development-release.yml`.
  `tools/dist-prepare/linux.py` only stages files, requires `chessvault.png` beside the binary, and rejects existing output.

## Package boundaries

- The binary-only `apps/chessvault` depends directly on `crates/chess-core` and `crates/platform-dirs`.
- `chess-core` exposes types through `src/lib.rs` re-exports (including `Move`); bitboards and castling rights stay internal.
  Preserve tested one-byte representations/discriminants. `Piece::Empty` and `Side::Empty` are invalid occupancy indices;
  `Square::None` is an invalid bit index.
- Construct public positions with `Position::standard()`; there is no public empty constructor or `Default`.
- `position/movegen.rs` owns legal move generation and application. `Position::play` rejects illegal moves without
  mutation; `Move` encodes castling with king squares and en passant with the pawn's landing square. Core generates
  all four promotions. Desktop must apply moves through `Game::play` to retain history and reject finished-game moves.
- `GameStatus::Checkmate(side)` and `Resigned(side)` identify the loser. Threefold/50-move draws require claims;
  fivefold/75-move draws are automatic. Intended-move claims do not play or record the move. Dead-position detection
  is conservative; blocked dead positions with pawns are not detected.

## State paths

- `platform-dirs` APIs only resolve paths; callers create directories.
  Default-enabled `development` plus debug assertions selects
  `<cwd>/.local/state` and `<cwd>/.local/share` on every platform; release builds or `default-features = false` use OS
  resolution on Linux and macOS (other platforms error).
- `system_share_dirs` always uses native resolution, even in development; it excludes user data directories.
  Preserve search order and duplicates; explicitly configured lists with no absolute entries return an empty vector.
- Linux and macOS require absolute XDG/home bases and preserve non-Unicode paths. Resolver tests inject environment/home
  values; do not mutate process-wide environment. Base resolution errors precede application-name validation.
- Path-policy changes also require `just check-platform-native` (the `platform-dirs` checks with
  `--no-default-features`); default debug checks do not exercise native resolution through the public entrypoint.
- Linux differential checks: `just --justfile venvs/justfile platform-dirs-differential`, then
  `just check-platform-differential`. This builds the native Rust probe and compares with pinned Python `platformdirs`;
  ordinary Rust tests need no Python. Review reference behavior changes when updating the pin.
- macOS state/data honor `XDG_STATE_HOME`/`XDG_DATA_HOME`, defaulting to `~/Library/Application Support`;
  system data honors `XDG_DATA_DIRS`, defaulting to `/Library/Application Support`.
- `user_log_dir` appends `log` unless its opinion argument is `Some(false)`, except native macOS uses
  `~/Library/Logs` and ignores opinion and XDG overrides.
