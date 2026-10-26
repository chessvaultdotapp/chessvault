# Chess core

`chess-core` implements chess positions, legal moves, standard algebraic notation
(SAN), game history, and results. It has no dependency on Iced, application runtime
helpers, or filesystem policy.

Use `Position` for board-level queries and `Game` for played games with history
and results. The desktop applies moves through `Game::play` and reconstructs
independent positions for read-only history review. PGN export belongs to the
desktop; there is no public core FEN loader or PGN parser.

Detailed documentation lives alongside the crate:

- [API overview](https://chessvaultdotapp.github.io/chessvault/dev/docs/crates/chess-core/)
- [Positions and moves](https://chessvaultdotapp.github.io/chessvault/dev/docs/crates/chess-core/positions/)
- [Games, history, and results](https://chessvaultdotapp.github.io/chessvault/dev/docs/crates/chess-core/games/)
- [Representation and invariants](https://chessvaultdotapp.github.io/chessvault/dev/docs/crates/chess-core/representation/)
- [Development and verification](https://chessvaultdotapp.github.io/chessvault/dev/docs/crates/chess-core/development/)

The [documentation sources](https://github.com/chessvaultdotapp/chessvault/tree/main/crates/chess-core/docs)
and package-local `zensical.toml` are under `crates/chess-core/`.
