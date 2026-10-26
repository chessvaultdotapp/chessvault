# Chess core

`chess-core` implements chess positions, legal moves, standard algebraic notation
(SAN), game history, and results. It is independent of Iced, application runtime
helpers, and filesystem policy.

## Public API

[`lib.rs`](../../crates/chess-core/src/lib.rs) re-exports `Position`, `Move`,
`PositionStatus`, `Game`, `GameStatus`, `DrawReason`, `Piece`, `Side`, `Square`,
and `InvalidSquareIndex`. Implementation modules, bitboards, and castling rights
remain private.

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

## Positions and moves

`Position` stores six piece-type bitboards and two side bitboards, plus side to
move, castling rights, an en passant target, and halfmove/fullmove counters.
Each occupied square must appear in exactly one piece board and one side board.
`piece_at` is a lookup, not a validator of internal occupancy consistency.

`Position::standard()` initializes the standard layout, White to move, all four
castling rights, no en passant target, a zero halfmove clock, and fullmove number
one. The crate-private `Position::empty()` is an internal construction helper,
not a legal starting position.

- `side_to_move`, `halfmove_clock`, and `fullmove_number` expose metadata.
- `legal_moves` generates legal moves, including castling, en passant, and all
  four promotions. Ordering is unspecified.
- `play` applies a legal move; an illegal move returns false without mutation.
- `san` formats a legal move without changing the position; illegal moves return `None`.
- `is_in_check` and `status` inspect board-based outcomes.

`Move` uses king origin/destination squares for castling and the pawn's landing
square for en passant. Promotion explicitly names a queen, rook, bishop, or knight.

## Games, history, and results

Use `Game::play` rather than changing a separate position when maintaining a game:
`Game` records moves and repetition history and rejects moves after a result.
`position()` exposes the live position read-only. `position_at(ply)` reconstructs
an independent position without rewinding the game; zero selects the starting position.

`PositionStatus` classifies ongoing play, checkmate, stalemate, and proven dead
material. `GameStatus` also accounts for resignation, agreement, and history-based
draws. `Checkmate(side)` and `Resigned(side)` identify the loser, not the winner.

Threefold repetition and the fifty-move rule require a claim. Fivefold repetition
and the seventy-five-move rule are automatic. Intended-move claims leave the board
and recorded history unchanged. Dead-position detection is conservative: bare
kings, a single minor piece, and bishops all on one colour are detected; blocked
dead positions with pawns are not.

Draw offers are made by the side to move and survive that player's move. Only
the opponent can accept or decline; the opponent's next legal move implicitly
declines. Invalid moves preserve the offer, and a game result clears it.

## Squares, pieces, and sides

| Type | Representation | Sentinel |
| --- | --- | --- |
| `Square` | One byte; A1–H1 are 0–7, continuing through A8–H8 at 56–63 | `None = 64` |
| `Piece` | One byte; Pawn, Knight, Bishop, Rook, Queen, King are 0–5 | `Empty = 6` |
| `Side` | One byte; White = 0, Black = 1 | `Empty = 2` |

For zero-based coordinates, a square index is `rank * 8 + file`.
`Square::try_from(u8)` accepts only 0–63, not the sentinel. Square display uses
variant spelling such as `E4`; SAN formatting supplies chess notation separately.
Sentinels are not valid occupancy indices or bit shifts. Tests pin representations
and discriminants, which also determine desktop artwork indexing.

Internal `Bitboard` wraps a `u64`, with A1 at bit zero and H8 at bit 63. It supports
empty construction, single-square conversion, membership, setting, and bitwise
operations. Invalid square conversion/setting fails; membership of `Square::None`
is false. Internal castling masks are 1/2 for White's kingside/queenside rights and
4/8 for Black's.

## Boundaries and verification

There is no public FEN loader or PGN parser. Finished-game headerless PGN export
belongs to the desktop, which replays core moves and requests SAN labels.

Tests live beside implementations and cover representation, move legality,
special moves, notation, history, draw rules, and result handling. Follow
[Testing](../development/testing.md) for ordered checks with `-p chess-core`;
use workspace checks for public APIs affecting the desktop. Markdown examples
are illustrative; put continuously tested examples in Rustdoc.
