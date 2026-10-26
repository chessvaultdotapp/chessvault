# Representation

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

## Position storage

`Position` stores six piece-type bitboards and two side bitboards, plus side to
move, castling rights, an en passant target, and halfmove/fullmove counters.
Each occupied square must appear in exactly one piece board and one side board.
A square's piece and side are found by consulting both sets of occupancy
bitboards. `piece_at` is a lookup, not a validator of internal occupancy consistency.

Internal `Bitboard` wraps a `u64`, with A1 at bit zero and H8 at bit 63. It supports
empty construction, single-square conversion, membership, setting, and bitwise
operations. Invalid square conversion/setting fails; membership of `Square::None`
is false. Internal castling masks are 1/2 for White's kingside/queenside rights and
4/8 for Black's.

These internals are not public construction APIs. Start with
[`Position::standard()`](positions.md) instead.
