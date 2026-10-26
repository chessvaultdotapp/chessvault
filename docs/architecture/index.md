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

Iced messages drive state changes; views read state and tasks perform effects.
Board moves go through `Game::play`, while review reconstructs a separate position
without mutating the live game. The renderer does not implement chess rules.
See the dedicated
[desktop architecture](https://chessvaultdotapp.github.io/chessvault/dev/docs/apps/chessvault/architecture/)
for startup, messages, UI guards, board rendering, and the source map.

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

The desktop owns the versioned window-size format and filesystem side effects;
the directory library only resolves paths. Chess positions are not persisted.
See the
[desktop runtime guide](https://chessvaultdotapp.github.io/chessvault/dev/docs/apps/chessvault/runtime/#window-persistence-and-failures)
for the binary format, startup fallbacks, and close/save behavior.

## Logging and developer console

The desktop installs the application's global tracing subscriber, collecting
application and library events for the console and optional per-launch JSON logs.
`RUST_LOG` controls capture; source toggles only filter retained snapshots.
See [desktop logging](https://chessvaultdotapp.github.io/chessvault/dev/docs/apps/chessvault/runtime/#developer-console)
for console behavior, event-store invariants, and file retention/failure policy.

## Tests and further reading

Unit tests live beside their implementations. They cover chess representations
and legal play, notation, history and results, log capture and filtering, window-state parsing,
and path resolution. Filesystem resolver tests inject inputs rather than
mutating process-wide environment variables; logging tests use scoped
subscribers instead of installing the application's global subscriber.

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for setup, ordered checks, and manual
verification expectations. API contracts belong in Rustdoc beside the code;
this page covers ownership and interactions across modules and packages.
