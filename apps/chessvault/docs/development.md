# Desktop development

Run commands from the workspace root with the pinned Rust toolchain and `--locked`.
Shared [setup](https://chessvaultdotapp.github.io/chessvault/dev/docs/development/getting-started/)
and [testing](https://chessvaultdotapp.github.io/chessvault/dev/docs/development/testing/)
remain the authority for tools, ordered checks, and cross-package changes.

## Run and debug

```sh
cargo run -p chessvault --locked
RUST_LOG=chessvault=debug cargo run -p chessvault --locked
```

Both commands launch a graphical application. F12 opens the console. Optional
`just run` and `just run-debug` provide shortcuts. Debug state is normally local
to the working directory; use a disposable directory for corrupt-state testing.
See [runtime state](runtime.md) for capture filtering and persistence failure policy.

## Artwork and icons

Piece artwork lives in `apps/chessvault/assets/rhosgfx-outline`. Preserve the
[artwork attribution and licensing](https://github.com/chessvaultdotapp/chessvault#acknowledgements).
Install the shared Node tooling with `pnpm install --frozen-lockfile`, then optimize
only artwork you intend to change:

```sh
pnpm exec svgo --input apps/chessvault/assets/rhosgfx-outline/wP.svg --output apps/chessvault/assets/rhosgfx-outline/wP.svg
git diff -- apps/chessvault/assets/rhosgfx-outline/wP.svg
```

This overwrites the SVG; review the diff, rebuild, and compare affected pieces
at multiple window sizes for unchanged shapes, colors, outlines, and scaling.
Assets are embedded at compile time. Cargo and CI do not automatically optimize
artwork; do not commit `node_modules`. See shared
[SVGO tooling](https://chessvaultdotapp.github.io/chessvault/dev/docs/tools/svgo/).

Edit `assets/logo.svg`, not generated icons. `build.rs` produces
`OUT_DIR/window-icon.rgba` (64×64 embedded fallback, straight/demultiplied RGBA)
and `OUT_DIR/chessvault.png` (distribution icon).
At startup `icon::load` searches `icons/hicolor/256x256/apps/chessvault.png`
under `user_share_dir(None)`, then each `system_share_dirs(None)` entry, in order.
The first decoded PNG wins. Missing files are skipped; unreadable/invalid icons
and resolution failures warn without preventing remaining searches. Exhaustion
uses the embedded fallback. Development user data uses `<cwd>/.local/share`;
system data lookup remains native.

## Automated checks

Use `just check-package chessvault` for desktop-only changes: build, nextest,
workspace-wide formatting, then Clippy. The binary-only package has no doctests.
Without nextest, use the shared Cargo-test fallback. For a focused window test:

```sh
cargo test -p chessvault --bin chessvault --locked window_recreate_tests::saved_dimensions_round_trip -- --exact
```

Inline tests cover application guards, promotion, history, logs, icon decoding,
path composition, and window format validation. They do not launch the GUI.
Use scoped tracing subscribers, not global `Logs::init()`, in tests.

## Verification

Visible changes require launching the app in a graphical session and exercising
the affected interactions, not merely checking startup:

- Resize: board stays square, centered, correctly oriented, and legible.
- Selection: only legal destinations appear; illegal clicks preserve position/history.
  Exercise castling and en passant when affected.
- Promotion: test queen, rook, bishop, and knight; Escape/outside click preserve
  position and leave the pawn selected.
- History: buttons, move clicks, Left/Right, Up/Home, Down/End select the expected
  ply; review is read-only and latest resumes play. Console/promotion block navigation.
- Results: checkmate, stalemate, resignation block play; result text and PGN
  respect losing-side semantics for checkmate/resignation.
- Offers: survive the offerer's move; opponent acceptance/decline and implicit
  decline by legal move work. Illegal moves preserve offers.
- Claims: current and intended-move claims work without recording intended moves.
  Exercise automatic fivefold, seventy-five-move, and supported dead-material results.
- New game: ongoing games with moves require confirmation. Cancel/Escape preserve
  state; accept resets history, selection, and offers.
- PGN: finished-game export has correct SAN/result; cancellation and disposable
  write failures preserve the game and allow retry.
- Console: F12, source filters, clear, copying, and selection during refresh work.
- Persistence: closing/reopening restores dimensions; test invalid state only in
  a disposable working directory.

Record checks and limitations in the PR, with screenshots when useful.

## Build this documentation

Following the package-local Zensical setup used by `platform-dirs`, this directory
has its own configuration and output under `apps/chessvault/site/`:

```sh
just --justfile venvs/justfile zensical
venvs/zensical/bin/zensical build --config-file apps/chessvault/zensical.toml
```

The documentation workflow builds this root and copies its output to
`dev/docs/apps/chessvault/` beside the shared and library sites. Cross-root links
use published URLs; relative links are reserved for pages within this root.
