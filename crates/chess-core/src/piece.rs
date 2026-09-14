/// A chess piece type, or a sentinel indicating that no piece is present.
///
/// Pawn through King have discriminants 0 through 5 for indexing the piece-type
/// bitboard array in [`crate::position::Position`].
/// [`Piece::Empty`] is not a valid index into that array.
#[repr(u8)]
pub(crate) enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    /// No piece; excluded from the piece-type bitboard array.
    Empty,
}

/// Number of piece types (six), excluding the [`Piece::Empty`] sentinel.
pub(crate) const NUM_PIECES: usize = 6;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn piece_uses_one_byte() {
        assert_eq!(size_of::<Piece>(), size_of::<u8>());
    }

    #[rstest]
    #[case::pawn(Piece::Pawn, 0)]
    #[case::knight(Piece::Knight, 1)]
    #[case::bishop(Piece::Bishop, 2)]
    #[case::rook(Piece::Rook, 3)]
    #[case::queen(Piece::Queen, 4)]
    #[case::king(Piece::King, 5)]
    #[case::empty(Piece::Empty, 6)]
    fn piece_variants_have_expected_values(#[case] variant: Piece, #[case] expected: u8) {
        assert_eq!(variant as u8, expected);
    }
}
