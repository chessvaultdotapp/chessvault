# Desktop application

The `chessvault` package is a binary-only Iced 0.14 application. It currently
renders the standard starting position, provides a developer console, and saves
window dimensions. The board is not yet interactive.

## Source map

| File | Responsibility |
| --- | --- |
| [`main.rs`](../../apps/chessvault/src/main.rs) | Startup, `ChessVault` state, messages, subscriptions, views, and window-state serialization |
| [`board.rs`](../../apps/chessvault/src/board.rs) | Responsive board layout and embedded piece artwork |
| [`logs.rs`](../../apps/chessvault/src/logs.rs) | Tracing setup, shared event storage, and source-filtered snapshots |
| [`fs.rs`](../../apps/chessvault/src/fs.rs) | Saved window-state file path |
| [`icon.rs`](../../apps/chessvault/src/icon.rs) | Installed window-icon lookup and embedded fallback |

## Application state and messages

`ChessVault` owns a `chess_core::Position`, a shared `Logs` handle, and console UI
state. The console keeps both the last displayed string and Iced's
`text_editor::Content`; the latter holds cursor and selection state.

The app uses `iced::application` with `boot`, `update`, `view`, and
`subscription`, not the older Iced `Application` trait API.

- `boot` creates the starting position and initializes the state directory.
- `update` changes state in response to `Message` values. Clipboard and window
  operations return Iced tasks; ordinary state updates return `Task::none()`.
- `view` derives the widget tree from application state.
- `subscription` listens for F12 and close requests, and adds a timer only when
  the console is open.

When adding an interaction, route it through messages and `update`. Keep board
presentation in `board.rs` and chess rules in `chess-core`, rather than giving
the renderer its own independent position representation.

## Board rendering and assets

`board::view` borrows a `Position` and queries `piece_at` for each square. Its
generic message type allows it to fit into the app without defining board
messages before there are interactions to handle.

The board uses the smaller available dimension as its side length. Ranks are
rendered from eight down to one and files from a through h, placing White at the
bottom. Rank labels occupy the left edge and file labels the bottom edge; a1 is
dark.

Piece SVGs live in
[`assets/rhosgfx-outline`](../../apps/chessvault/assets/rhosgfx-outline). They are
embedded with `include_bytes!` and converted into cached handles through
`LazyLock`. This avoids runtime asset-path dependencies and reuses renderer
cache entries. The handle table follows `Side` and `Piece` discriminants;
checked indexing excludes their empty sentinels.

See the [README acknowledgements](../../README.md#acknowledgements) for artwork
attribution and licensing.

### Window icon lookup

At startup, [`icon::load`](../../apps/chessvault/src/icon.rs) searches for
`icons/hicolor/256x256/apps/chessvault.png` beneath these data directories, in order:

1. The user data directory returned by `platform_dirs::user_share_dir(None)`.
2. The system data directories returned by `platform_dirs::system_share_dirs(None)`.

On Linux, native user-directory resolution uses an absolute `XDG_DATA_HOME`, or
`$HOME/.local/share` otherwise. System directories follow `XDG_DATA_DIRS` in order,
with `/usr/local/share` then `/usr/share` as the default when it is unset or empty.
In debug builds with the default `development` feature, the user directory is
`<cwd>/.local/share`; system directories still use native resolution. See
[platform directory resolution](platform-dirs.md) for the full path policy.

The first successfully decoded PNG wins. Missing icons are skipped; unreadable or
invalid icons produce a warning and the search continues. Directory-resolution
failures also produce warnings without preventing the remaining search. If no
installed icon can be loaded, the app uses the embedded 64×64 window icon generated
from [`assets/logo.svg`](../../apps/chessvault/assets/logo.svg) at build time.

### Optimizing SVG artwork

Use the optional [SVGO tooling](../tools/svgo.md)
when adding or changing piece artwork. SVGO is declared in
the root [`package.json`](../../package.json), with dependency versions
recorded in [`pnpm-lock.yaml`](../../pnpm-lock.yaml). It is not
invoked automatically by Cargo or CI.

From the workspace root, optimize a changed SVG (here, the white pawn). The
following output examples are abbreviated; timings, sizes, and build messages
vary. An already optimized SVG may show no size reduction or diff.

```console
$ pnpm install --frozen-lockfile
Lockfile is up to date, resolution step is skipped
Already up to date
...
$ pnpm exec svgo --input apps/chessvault/assets/rhosgfx-outline/wP.svg --output apps/chessvault/assets/rhosgfx-outline/wP.svg
wP.svg:
Done in 23 ms!
1.282 KiB - 0% = 1.282 KiB
$ git diff -- apps/chessvault/assets/rhosgfx-outline/wP.svg
```

This overwrites the input file; run it only on artwork you intend to update.
Review the diff and preserve artwork attribution and licensing. Then rebuild
and launch the app in a graphical session:

```console
$ cargo run -p chessvault --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running `target/debug/chessvault`
...
```

Compare the affected pieces before and after optimization, including at different
window sizes. Check that shapes, colors, outlines, and scaling are unchanged.
The SVGs are embedded at compile time, so a running app will not pick up asset
edits until rebuilt. Commit the optimized SVGs with any intentional dependency
changes; do not commit `node_modules`.

## Developer console

Press **F12** to open or close the console. To capture app debug messages, run
from the workspace root (example output abbreviated):

```console
$ RUST_LOG=chessvault=debug cargo run -p chessvault --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running `target/debug/chessvault`
...
```

`RUST_LOG` filters capture, not just display. A message excluded at this stage
cannot be recovered by enabling a source in the console. Without a valid
`RUST_LOG` configuration, capture defaults to `off,chessvault=info,platform_dirs=info`:
application and platform-dirs logs at info level and above are captured. Other dependency
logs are excluded from console capture, terminal output, and log files.

The source controls operate on retained events. The application source is enabled
initially, and `platform_dirs` is enabled when discovered. Other new sources are
hidden until explicitly enabled. “Copy all” copies
the filtered snapshot, while “Clear” removes all retained events, including
hidden ones, without changing source settings.

### Invariants to preserve

- Install the global subscriber once, before GUI startup. Tests use scoped
  subscribers instead.
- Retain at most 2,000 complete events, not lines. Publish each event atomically
  through `Logs::store_event` so concurrent formatting cannot interleave fragments.
- Normalize bridged `log` records before determining their source. Their static
  tracing target alone would incorrectly group dependency messages under `log`.
- Reject text-editor editing actions while accepting navigation and selection.
- Skip periodic display refresh while text is selected. Capture continues in
  the shared store; selection should not be lost merely because a timer fires.
- Run the 250 ms refresh subscription only while the console is open.

Terminal logging is enabled only with debug assertions. Both debug and release
builds capture events for the in-app console and attempt to write a per-launch
JSON log file under `user_log_dir`. If resolving the directory or opening the
file fails, startup continues without file logging and emits a warning through
the remaining logging outputs, subject to `RUST_LOG`. Failure to install the
global logging subscriber still aborts startup. Console truncation notices do
not assume a full event is available on disk.

After opening the current log and installing the subscriber, a background thread
cleans up logs without delaying GUI startup. It keeps at most 20 files, including
the current log, and deletes logs older than 7 days based on their filename
timestamps. The current log is always preserved. Only regular files matching the
exact application log filename format are eligible; unrelated files, directories,
and symlinks are untouched. Cleanup failures are logged at error level, subject
to `RUST_LOG`, and do not abort startup or stop other deletions where possible.
Cleanup runs once per launch, not periodically. There is no cross-instance locking;
other instances' active logs are not protected.

## Window persistence and failures

`WindowRecreateInfo` loads and saves JSON containing width and height. Loading
rejects non-positive or non-finite dimensions. A missing file uses the default
window size; read or parse failures are logged and also use the default.

The app disables automatic exit on close requests. It first queries the window
size, attempts to save it, and then closes even if saving failed. In contrast,
failure to create the application state directory during boot is fatal.

Saving directly truncates the `recreate` file; writes are not atomic. To reset
window state, remove that file while the app is closed. See
[Platform directories](platform-dirs.md) for application paths, platform policy, and limitations.

## Verification

Follow the ordered checks in [Testing](../development/testing.md), selecting
`-p chessvault` for desktop-only changes. Inline tests cover logs, file-path
composition, and saved-dimension parsing; they do not replace GUI verification.

For affected visible interactions, launch the app and check:

- Resizing keeps the board square, centered, and correctly oriented.
- F12 opens and closes the console; filtering and copying behave as expected.
- Selected log text remains selected during periodic refresh.
- Closing and reopening restores the saved window size.

Run GUI checks in a disposable working directory when testing invalid saved
state, rather than damaging state you want to keep.
