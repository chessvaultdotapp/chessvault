# Architecture and UI flow

## Source map

Source links below refer to the repository's `main` branch.

| Module | Responsibility |
| --- | --- |
| [main.rs](https://github.com/chessvaultdotapp/chessvault/blob/main/apps/chessvault/src/main.rs) | Startup, `ChessVault`, messages, views, subscriptions, and window serialization |
| [board.rs](https://github.com/chessvaultdotapp/chessvault/blob/main/apps/chessvault/src/board.rs) | Selection, legal destinations, board layout, and embedded pieces |
| [history.rs](https://github.com/chessvaultdotapp/chessvault/blob/main/apps/chessvault/src/history.rs) | Cached SAN labels and independent review positions |
| [game_status.rs](https://github.com/chessvaultdotapp/chessvault/blob/main/apps/chessvault/src/game_status.rs) | Human-readable results |
| [pgn.rs](https://github.com/chessvaultdotapp/chessvault/blob/main/apps/chessvault/src/pgn.rs) | Finished-game headerless SAN movetext export |
| [logs.rs](https://github.com/chessvaultdotapp/chessvault/blob/main/apps/chessvault/src/logs.rs) | Tracing setup, event storage, and filtered snapshots |
| [fs.rs](https://github.com/chessvaultdotapp/chessvault/blob/main/apps/chessvault/src/fs.rs) | Saved window-state path |
| [icon.rs](https://github.com/chessvaultdotapp/chessvault/blob/main/apps/chessvault/src/icon.rs) | Installed icon lookup and embedded fallback |
| [serialization](https://github.com/chessvaultdotapp/chessvault/tree/main/apps/chessvault/src/serialization) | Versioned binary headers |

## Lifecycle and ownership

Startup installs the global tracing subscriber, loads window dimensions with a
fallback to defaults, and boots `ChessVault` with `Game::new(Position::standard())`.
Creating the application state directory during boot is fatal on failure.

The app uses `iced::application`, not the older `Application` trait.
`ChessVault` owns the live `Game`, independent `History`, selection, pending
promotion, new-game confirmation, PGN-save state, shared `Logs`, and console UI state.

- `update` handles `Message` values and returns tasks for effects such as clipboard,
  save dialogs, and window operations; ordinary updates return `Task::none()`.
- `view` derives widgets from state. Keep board presentation in `board.rs` and chess rules in `chess-core`.
- `subscription` handles navigation keys, F12, Escape, and close requests.
  A 250 ms timer is subscribed only while the console is open.

## Board, moves, and review

`board::view` reads the live or independently reconstructed review `Position`;
it does not maintain another chess representation. Clicks become messages, then
selection changes or legal moves through `Game::play`, preserving history and results.
Do not apply desktop moves directly to `Position`.

Promotion waits for an on-board queen/rook/bishop/knight choice. Escape or an
outside click cancels without changing the position and keeps the pawn selected.
Other board moves are blocked while choosing. History review never rewinds the
live game; return to latest to resume play. Navigation is blocked while the
console or promotion picker is open. SAN labels are cached when moves arrive,
not on redraw.

Resignation, draw offers/responses, current-position and intended-move draw claims
delegate to the core. Intended claims do not play or record the move. Result
variants for checkmate and resignation identify the losing side.
New game requests confirmation before discarding an ongoing game with recorded moves.
Finished games enable Save PGN: `pgn::export` replays from `Game::position_at(0)`
and emits headerless SAN movetext with the result token, not a full PGN database.

## Rendering

The board uses the smaller available dimension, White at the bottom, ranks eight
to one and files a to h. Rank labels are left, file labels below, and a1 is dark.
Piece SVGs in `assets/rhosgfx-outline` are embedded with `include_bytes!` and
cached as `LazyLock` handles. Rendering is independent of the working directory
and reuses renderer cache entries. Checked indexing excludes `Side::Empty` and
`Piece::Empty`; the table follows their discriminants.

See [development](development.md#artwork-and-icons) for artwork changes and
[runtime state](runtime.md) for persistence and console behavior.
