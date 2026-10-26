# Architecture

This page describes the current implementation, rather than a target design.
Chessvault is a Rust workspace with an Iced desktop application and two
supporting libraries. The app supports local chess play, legal-move hints,
promotion selection, read-only move-history review, game results, and finished-game
PGN export. It also provides an in-app developer console and remembers its window size.

## Package boundaries

```text
chessvault (desktop binary)
├── chess-core
└── platform-dirs
```

Arrows in this tree represent workspace dependencies; third-party dependencies
are omitted.

| Package | Responsibility | Entry point |
| --- | --- | --- |
| `chessvault` | UI, application state, logging, and window-size persistence | [`apps/chessvault/src/main.rs`](../../apps/chessvault/src/main.rs) |
| `chess-core` | Positions, legal moves, SAN, game history, and results | [`crates/chess-core/src/lib.rs`](../../crates/chess-core/src/lib.rs) |
| `platform-dirs` | Platform-specific user state, share, and log directory resolution | [`crates/platform-dirs/src/lib.rs`](../../crates/platform-dirs/src/lib.rs) |

The chess core has no dependency on the GUI or platform directory library.
Platform resolution accepts an optional application name; the desktop passes
`chessvault`. The desktop owns filesystem side effects and its saved window-state format.

## Desktop lifecycle and state

The binary uses Iced 0.14's `iced::application` builder. `ChessVault` owns the
live `Game`, independent `History` review state, selection and promotion state,
shared log store, and console presentation state.

Startup proceeds as follows:

1. Install the global tracing subscriber so startup failures can be captured.
2. Try to load the saved window dimensions. Missing state uses Iced's default
   size; invalid or unreadable state logs an error and also falls back.
3. Boot `ChessVault` with a `Game` constructed from `Position::standard()` and the console closed.
4. Create the application state directory. Failure here is fatal.

Iced drives three methods:

- `update` handles messages and returns tasks for effects such as clipboard
  writes and window operations.
- `view` builds the board and, when open, the developer console.
- `subscription` listens for keyboard navigation, F12, Escape, and window-close
  requests, adding a 250 ms log refresh timer only while the console is open.

Automatic closing is disabled. On a close request, the app queries the window
size, attempts to save it, and then closes the window even if saving fails.

### Board rendering

[`board.rs`](../../apps/chessvault/src/board.rs) takes an immutable `Position` and
queries `Position::piece_at` for each square. It does not own a separate copy of
the chess state or implement chess rules.

The board scales to the available space, with White at the bottom and a1 dark.
Piece SVGs are embedded in the binary, so rendering does not depend on the
working directory. Their handles are cached and reused across redraws.

Board messages update selection or apply moves through `Game::play`. Promotion
waits for an explicit piece choice. History review reconstructs an independent
position and never mutates the live game; returning to latest resumes play.
SAN labels are cached when moves arrive. New-game and PGN-save actions are also
handled by the application, not the renderer.

## Chess core integration

`Position` generates and applies legal moves and formats SAN. `Game` adds move
and repetition history, resignation, draw offers, claim-based draws, and automatic
results. Desktop moves must use `Game::play`, which rejects finished-game moves.
See the [core overview](../components/chess-core.md) for package documentation,
including representation invariants and rule boundaries. The desktop owns
finished-game SAN movetext export rather than the core.

## Runtime and persistence

State paths are resolved in layers:

```text
platform_dirs::user_state_dir(Some("chessvault"))      [append chessvault to base]
    → desktop fs::window_recreate_info_filepath()    [append recreate]
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

The library also exposes `user_share_dir` for persistent user data. Its development
base is `<current working directory>/.local/share`; native Linux resolution uses
an absolute `XDG_DATA_HOME`, falling back to `<home>/.local/share`. It follows the
same application-name validation and platform support as state resolution.
The desktop uses this API, followed by native `system_share_dirs`, to search for
installed window icons before falling back to embedded pixels.

### Saved data and failure handling

The `recreate` file is a 24-byte binary record with a versioned header and two
little-endian `f32` dimensions. Loading requires the exact supported header and
length and finite, positive dimensions. Legacy JSON is rejected, logged, and
falls back to defaults rather than being migrated. See the
[desktop format description](../components/desktop.md#window-persistence-and-failures).
It does not restore window position or save the chess position.

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
  `off,chessvault=info,platform_dirs=info` (application and platform-dirs logs). The same filter applies to log
  files and terminal output. Terminal output is enabled only in debug builds; in-app capture also
  works in release builds.
- **Display:** console source toggles filter snapshots of retained events.
  Hidden events can be revealed later, until cleared or evicted.

Sources are the first `::`-separated component of an event target. Bridged `log`
records are normalized to preserve the original dependency source.

F12 toggles the console. Its text editor rejects editing actions, and periodic
refresh pauses while text is selected so copying remains usable. “Copy all”
copies the filtered snapshot. Clearing history preserves source settings.
Console history and source settings are not restored between launches. Captured
events are also written to a per-launch JSON log when file initialization succeeds;
background retention preserves the current log and prunes eligible old logs. See
[desktop logging](../components/desktop.md#developer-console) for failure and retention policy.

## Tests and further reading

Unit tests live beside their implementations. They cover chess representations
and legal play, notation, history and results, log capture and filtering, window-state parsing,
and path resolution. Filesystem resolver tests inject inputs rather than
mutating process-wide environment variables; logging tests use scoped
subscribers instead of installing the application's global subscriber.

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for setup, ordered checks, and manual
verification expectations. API contracts belong in Rustdoc beside the code;
this page covers ownership and interactions across modules and packages.
