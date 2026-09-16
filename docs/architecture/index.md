# Architecture

This page describes the current implementation, rather than a target design.
Chessvault is a Rust workspace with an Iced desktop application and three
supporting libraries. The app displays the standard chess starting position,
provides an in-app developer console, and remembers its window size. It does not
yet support playing moves.

## Package boundaries

```text
chessvault (desktop binary)
├── chess-core
└── application-runtime
    └── platform-dirs
```

Arrows in this tree represent workspace dependencies; third-party dependencies
are omitted.

| Package | Responsibility | Entry point |
| --- | --- | --- |
| `chessvault` | UI, application state, logging, and window-size persistence | [`apps/chessvault/src/main.rs`](../../apps/chessvault/src/main.rs) |
| `chess-core` | Chess primitives and bitboard-based position storage | [`crates/chess-core/src/lib.rs`](../../crates/chess-core/src/lib.rs) |
| `application-runtime` | Application-specific runtime helpers; currently the Chessvault state-directory path | [`crates/application-runtime/src/lib.rs`](../../crates/application-runtime/src/lib.rs) |
| `platform-dirs` | Platform-specific user state-directory resolution | [`crates/platform-dirs/src/lib.rs`](../../crates/platform-dirs/src/lib.rs) |

The chess core has no dependency on the GUI or runtime libraries. Platform
resolution does not know the application name; `application-runtime` adds that
policy. The desktop application owns filesystem side effects and the format of
its saved window state.

## Desktop lifecycle and state

The binary uses Iced 0.14's `iced::application` builder. `ChessVault` owns the
current `Position`, shared log store, and console presentation state.

Startup proceeds as follows:

1. Install the global tracing subscriber so startup failures can be captured.
2. Try to load the saved window dimensions. Missing state uses Iced's default
   size; invalid or unreadable state logs an error and also falls back.
3. Boot `ChessVault` with `Position::standard()` and the console closed.
4. Create the application state directory. Failure here is fatal.

Iced drives three methods:

- `update` handles messages and returns tasks for effects such as clipboard
  writes and window operations.
- `view` builds the board and, when open, the developer console.
- `subscription` listens for F12 and window-close requests, adding a 250 ms log
  refresh timer only while the console is open.

Automatic closing is disabled. On a close request, the app queries the window
size, attempts to save it, and then closes the window even if saving fails.

### Board rendering

[`board.rs`](../../apps/chessvault/src/board.rs) takes an immutable `Position` and
queries `Position::piece_at` for each square. It does not own a separate copy of
the chess state or implement chess rules.

The board scales to the available space, with White at the bottom and a1 dark.
Piece SVGs are embedded in the binary, so rendering does not depend on the
working directory. Their handles are cached and reused across redraws.

There are currently no board-interaction messages or move-making operations.

## Chess representation

`chess-core` publicly exports `Position`, `Piece`, `Side`, `Square`, and
`InvalidSquareIndex`; its implementation modules remain private.

A [`Position`](../../crates/chess-core/src/position.rs) stores:

- Six piece-type bitboards, one each for pawn through king.
- Two side bitboards, one each for White and Black.
- Side to move, castling-rights flags, and halfmove and fullmove counts.

A square's piece and side are found by consulting both sets of occupancy
bitboards. `Piece::Empty` and `Side::Empty` are sentinels, not valid indices into
these arrays. `Square::None` represents no square and is not a valid bit index.

`Position::standard()` constructs the normal starting position with White to
move, all castling rights, a zero halfmove clock, and fullmove number one.
`piece_at` returns `None` for an empty square or `Square::None`.
The crate-internal `Position::empty()` instead clears all occupancy and counts,
with no side to move or castling rights; it is not a playable starting position.

The core currently provides representation and lookup, not legal move
generation, move execution, game-result detection, or FEN/PGN support. Those
capabilities should not be inferred from the presence of position metadata.

## Runtime and persistence

State paths are resolved in layers:

```text
platform_dirs::user_state_dir()
    → application_runtime::fs::application_state_dir()  [append chessvault]
    → desktop fs::window_recreate_info_filepath()        [append recreate]
```

The path helpers return `PathBuf` values without creating directories and
preserve non-Unicode paths. The desktop app creates the application directory
and reads or writes the `recreate` file.

### Directory policy

With `platform-dirs`' default-enabled `development` feature, debug builds use
`<current working directory>/.local/state` on every platform. When launched from
the workspace root, saved window state therefore lives at
`.local/state/chessvault/recreate`.

Release builds, and builds without that feature, use platform resolution:

- On Linux, an absolute `XDG_STATE_HOME` takes precedence. Otherwise an absolute
  home directory is required and `.local/state` is appended.
- Other platforms currently return an error. The development path does not mean
  that native state-directory support exists for those platforms.

### Saved data and failure handling

The `recreate` file contains JSON with window `width` and `height`. Loading
requires finite, positive dimensions. It does not restore window position or
save the chess position.

Saving currently creates or truncates the file directly; it is not an atomic
replace. A damaged file is handled by the startup fallback described above.
Filesystem errors retain their underlying causes with context supplied by each
layer.

## Logging and developer console

[`logs.rs`](../../apps/chessvault/src/logs.rs) installs a tracing subscriber and
captures formatted events in a shared `Arc<Mutex<LogStore>>`. The store retains
the latest 2,000 complete events, not 2,000 lines. Each event is collected before
being published so concurrent writers cannot interleave fragments.

There are two distinct filtering stages:

- **Capture:** `RUST_LOG` controls which events enter the store, defaulting to
  `info`. Terminal output is enabled only in debug builds; in-app capture also
  works in release builds.
- **Display:** console source toggles filter snapshots of retained events.
  Hidden events can be revealed later, until cleared or evicted.

Sources are the first `::`-separated component of an event target. Bridged `log`
records are normalized to preserve the original dependency source.

F12 toggles the console. Its text editor rejects editing actions, and periodic
refresh pauses while text is selected so copying remains usable. “Copy all”
copies the filtered snapshot. Clearing history preserves source settings.
Logs and source settings are not persisted to disk.

## Tests and further reading

Unit tests live beside their implementations. They cover chess representations
and starting-position lookup, log capture and filtering, window-state parsing,
and path resolution. Filesystem resolver tests inject inputs rather than
mutating process-wide environment variables; logging tests use scoped
subscribers instead of installing the application's global subscriber.

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for setup, ordered checks, and manual
verification expectations. API contracts belong in Rustdoc beside the code;
this page covers ownership and interactions across modules and packages.
