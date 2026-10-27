# Desktop guide

Paths below are relative to `apps/chessvault/`; run commands from the workspace root.

## Run and verify

- Launch the GUI: `just run` or `cargo run -p chessvault --locked`; `just run-debug` enables app debug logs.
- Visible changes require launching in a graphical session and exercising the affected interaction, not just startup.
  See `apps/chessvault/docs/development.md` for the interaction checklist.
- Single unit test (does not launch the GUI):
  `cargo test -p chessvault --bin chessvault --locked window_recreate_tests::saved_dimensions_round_trip -- --exact`.
  Full package verification: `just check-package chessvault`.

## App wiring

- `src/main.rs` uses Iced **0.14**'s `iced::application` builder with `ChessVault::{update, view, subscription}`.
  Follow this API rather than older Iced `Application` trait examples.
- `ChessVault` owns `Game`, `History`, and selection; `src/board.rs` calls core move APIs.
- History review reconstructs a separate position; never rewind or mutate the live game. Review is read-only;
  navigation is blocked while the console or promotion picker is open. Cache SAN labels when moves arrive, not on redraw.
- Promotion is deferred until the on-board picker chooses a piece. Outside clicks or Escape keep the pawn selected
  without changing the position; board moves are blocked while choosing.
- `.exit_on_close_request(false)` lets the close subscription query and save window size before `window::close`. Restore
  failures use defaults; save failures still close the window.
- Desktop state is normally `.local/state/chessvault/` relative to the workspace root in development;
  `recreate` (no extension) is a versioned 24-byte binary window-size file, not JSON.

## Assets

- Preserve embedded SVGs (`include_bytes!`) and cached `LazyLock` handles so rendering is directory-independent
  and reuses handles.
- `build.rs` renders `assets/logo.svg` into `OUT_DIR/window-icon.rgba` (embedded by `src/icon.rs`) and
  `OUT_DIR/chessvault.png` (distribution icon). Edit the SVG source; window pixels must use straight (demultiplied) RGBA.

## Logging and console

- The **F12** console is a read-only `text_editor`; preserve selection by skipping periodic refresh while text is selected.
- `Logs::init()` installs the global tracing subscriber before the GUI starts. Tests instead use scoped subscribers via
  `tracing::subscriber::with_default`.
- `RUST_LOG` controls all logging (default `off,chessvault=info,platform_dirs=info`). Terminal output requires debug assertions;
  in-app capture works in release.
- Log-file initialization and background retention failures are nonfatal; subscriber installation failure is fatal.
  Retention must preserve the current log, unrelated files, and symlinks; there is no cross-instance locking.
- Console source toggles filter snapshots, not capture; `clear()` preserves source settings and “Copy all” uses the
  filtered snapshot. Preserve `NormalizeEvent` for bridged `log` records. Publish complete events atomically through
  `Logs::store_event`; retention is 2,000 events, not lines.
