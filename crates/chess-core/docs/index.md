# Chess core

`chess-core` implements chess positions, legal moves, standard algebraic notation
(SAN), game history, and results. It is independent of Iced, application runtime
helpers, and filesystem policy.

## Public API

[`lib.rs`](https://github.com/chessvaultdotapp/chessvault/blob/main/crates/chess-core/src/lib.rs)
re-exports `Position`, `Move`, `PositionStatus`, `Game`, `GameStatus`, `DrawReason`,
`Piece`, `Side`, `Square`, and `InvalidSquareIndex`. Implementation modules,
bitboards, and castling rights remain private.

Construct public positions with `Position::standard()`; there is no `Default`
or arbitrary-position loader. For example:

```rust
use chess_core::{Game, Move, Position, Square};

let mut game = Game::new(Position::standard());
assert!(game.play(Move {
    from: Square::E2,
    to: Square::E4,
    promotion: None,
}));
assert_eq!(game.moves().len(), 1);
```

## Guides

- [Positions and moves](positions.md): construction, legal moves, special moves, and SAN.
- [Games and results](games.md): history, result semantics, draw claims, and offers.
- [Representation](representation.md): compact types, sentinels, and internal occupancy invariants.
- [Development](development.md): boundaries, tests, and documentation builds.

There is no public FEN loader or PGN parser. Finished-game headerless PGN export
belongs to the desktop, which replays core moves and requests SAN labels.

See the [shared component overview](https://chessvaultdotapp.github.io/chessvault/dev/docs/components/chess-core/)
for workspace integration.
