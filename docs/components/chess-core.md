# Chess core

`chess-core` contains chess primitives and bitboard-based position storage. It
is independent of Iced, application runtime helpers, and filesystem policy.
It currently supports constructing the standard position and looking up pieces,
not playing or validating moves.

## Public API

[`lib.rs`](../../crates/chess-core/src/lib.rs) re-exports `Position`, `Piece`, `Side`,
`Square`, and `InvalidSquareIndex`. The underlying modules are private;
`Bitboard` and `CastlingRights` are internal implementation details.

A consumer can inspect the starting position:

```rust
use chess_core::{Piece, Position, Side, Square};

let position = Position::standard();
assert_eq!(position.piece_at(Square::E1), Some((Piece::King, Side::White)));
assert_eq!(position.piece_at(Square::E4), None);
assert_eq!(position.piece_at(Square::None), None);
```

Position fields are private. There is currently no public mutation API, metadata
getter API, arbitrary-position loader, or `Default` implementation for `Position`.

## Squares, pieces, and sides

| Type | Representation | Sentinel |
| --- | --- | --- |
| [`Square`](../../crates/chess-core/src/square.rs) | One byte; A1–H1 are 0–7, continuing through A8–H8 at 56–63 | `None = 64` |
| [`Piece`](../../crates/chess-core/src/piece.rs) | One byte; Pawn, Knight, Bishop, Rook, Queen, King are 0–5 | `Empty = 6` |
| [`Side`](../../crates/chess-core/src/side.rs) | One byte; White = 0, Black = 1 | `Empty = 2` |

For zero-based rank and file coordinates, the square index is `rank * 8 + file`.
`Square::try_from(u8)` accepts only 0–63; even the sentinel index 64 returns
`InvalidSquareIndex`. Use `Square::None` explicitly when representing absence.
Square display currently uses variant spelling such as `E4`, not lowercase
algebraic notation.

The sentinels represent absence, not additional pieces, sides, or board squares.
Never use them as occupancy-array indices or shift counts. Tests pin both the
one-byte representations and discriminant values; changing enum order affects
storage and consumers such as the desktop artwork table.

## Bitboards

[`Bitboard`](../../crates/chess-core/src/bitboard.rs) wraps a private `u64`. Bit zero
corresponds to A1 and bit 63 to H8. A set bit means a square belongs to the set.

Internal operations include:

- `empty()` and `Default`: a zero-bit set; prefer `empty()` for explicit
  construction.
- `TryFrom<Square>`: a single-square set, rejecting `Square::None`.
- `set`: add a square without changing other bits; invalid input returns an
  error without modifying the board.
- `contains`: membership lookup, returning false for `Square::None`.
- Bitwise AND, OR, and XOR, including their assignment forms.

A bitboard is only a set of squares. Its role as piece-type or side occupancy
comes from its location in a `Position`.

## Position storage and consistency

[`Position`](../../crates/chess-core/src/position.rs) stores six piece-type
bitboards and two side bitboards, rather than a separate object per occupied
square. A white pawn, for example, contributes a bit to both the pawn board and
the White board.

For a consistent position, each occupied square must belong to exactly one
piece-type board and exactly one side board, and both groups must describe the
same overall occupancy. These are representation invariants, not a claim that
the position is legally reachable.

`piece_at` searches side and piece-type occupancy and returns the matching pair.
It is a lookup, not a validator: it does not report overlapping boards or other
internal inconsistencies. Future mutation code must preserve consistency in
both groups of bitboards.

Additional private fields hold side to move, a `u8` castling-rights mask, a `u8`
halfmove count, and a `u32` fullmove count. Internal castling masks are 1 and 2
for White's kingside and queenside rights, and 4 and 8 for Black's; OR-combining
them gives all rights (15).

### Construction

| State | `Position::standard()` | Internal `Position::empty()` |
| --- | --- | --- |
| Occupancy | Standard 32-piece layout | All boards zero |
| Side to move | White | `Side::Empty` |
| Castling rights | All four | None |
| Halfmove count | 0 | 0 |
| Fullmove count | 1 | 0 |

The empty constructor is crate-visible, not part of the public API, and does
not create a legal starting position. The standard constructor uses it as a
base before populating occupancy and metadata.

## Current boundaries

There is no move generation, move execution, check detection, game-result
detection, or FEN/PGN support yet. Position storage also has no en passant field.
Castling rights and clocks are currently initialized data, not evidence of
implemented rules or automatic updates.

Keep new chess behavior independent of GUI and storage concerns. Expose only
the operations consumers need, and document sentinel handling and consistency
requirements beside the API in Rustdoc.

## Verification

Tests live beside each implementation. They cover representation sizes and
values, square conversion, bit operations, empty construction, starting-position
masks and metadata, and square lookup across piece/side combinations.

Follow [Testing](../development/testing.md) for ordered checks, using
`-p chess-core` for core-only changes. Use workspace checks when a public API
change also changes the desktop consumer. Markdown examples are illustrative;
place examples that need continuous doctest coverage in Rustdoc.
