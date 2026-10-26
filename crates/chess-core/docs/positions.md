# Positions and moves

`Position::standard()` initializes the standard layout, White to move, all four
castling rights, no en passant target, a zero halfmove clock, and fullmove number
one. The crate-private `Position::empty()` is an internal construction helper,
not a legal starting position: it clears occupancy and counters, with no side to
move or castling rights.

- `piece_at` returns `None` for an empty square or `Square::None`.
- `side_to_move`, `halfmove_clock`, and `fullmove_number` expose metadata.
- `legal_moves` generates legal moves, including castling, en passant, and all
  four promotions. Ordering is unspecified.
- `play` applies a legal move; an illegal move returns false without mutation.
- `san` formats a legal move without changing the position; illegal moves return `None`.
- `is_in_check` and `status` inspect board-based outcomes.

`Move` uses king origin/destination squares for castling and the pawn's landing
square for en passant. Promotion explicitly names a queen, rook, bishop, or knight.

`Position` does not retain game history. Use [Game](games.md) when moves must be
recorded or history-dependent results enforced. See [Representation](representation.md)
for occupancy invariants rather than treating `piece_at` as a validator.
