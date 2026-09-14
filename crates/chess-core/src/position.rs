use crate::{
    bitboard::Bitboard,
    castling_rights::CastlingRights,
    piece::NUM_PIECES,
    side::{NUM_SIDES, Side},
};

/// Occupancy bitboards, side to move, castling rights, and move counts.
pub(crate) struct Position {
    /// Six piece-type bitboards, indexed by Pawn through King.
    /// The [`Piece::Empty`](crate::piece::Piece::Empty) sentinel has no entry and is not a valid index.
    pieces: [Bitboard; NUM_PIECES],
    /// Two side bitboards, indexed by White and Black.
    /// The [`Side::Empty`] sentinel has no entry and is not a valid index.
    sides: [Bitboard; NUM_SIDES],
    /// The side whose turn it is, or [`Side::Empty`] when no side is to move.
    side_to_move: Side,
    /// Bitwise OR of [`CastlingRights`] values cast to `u8`.
    castling_rights: u8,
    /// Number of halfmoves since the last pawn move or capture.
    halfmove_count: u8,
    /// Full move count, initialized to zero in an empty position.
    fullmove_count: u32,
}

impl Position {
    /// Creates an empty position with no pieces, side to move, or castling rights.
    ///
    /// All six piece-type bitboards and both side bitboards are initialized
    /// with [`Bitboard::empty`], and the side to move is [`Side::Empty`].
    /// Neither array includes an entry for its `Empty` sentinel.
    /// Both move counts are initialized to zero.
    pub(crate) fn empty() -> Self {
        Self {
            pieces: std::array::from_fn(|_| Bitboard::empty()),
            sides: std::array::from_fn(|_| Bitboard::empty()),
            side_to_move: Side::Empty,
            castling_rights: CastlingRights::NoCastling as u8,
            halfmove_count: 0,
            fullmove_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::Piece;
    use rstest::{fixture, rstest};

    #[rstest]
    #[case::pawn(Piece::Pawn)]
    #[case::knight(Piece::Knight)]
    #[case::bishop(Piece::Bishop)]
    #[case::rook(Piece::Rook)]
    #[case::queen(Piece::Queen)]
    #[case::king(Piece::King)]
    fn empty_position_has_one_zero_bitboard_per_piece_type(#[case] piece: Piece) {
        let position = Position::empty();

        assert_eq!(position.pieces.len(), 6);
        assert_eq!(position.pieces[piece as usize], Bitboard::empty());
    }

    #[rstest]
    #[case::white(Side::White)]
    #[case::black(Side::Black)]
    fn empty_position_has_one_zero_bitboard_per_side(#[case] side: Side) {
        let position = Position::empty();

        assert_eq!(position.sides.len(), 2);
        assert_eq!(position.sides[side as usize], Bitboard::empty());
    }

    #[fixture]
    fn empty_position() -> Position {
        Position::empty()
    }

    #[rstest]
    #[case::no_side_to_move(|position: &Position| assert!(matches!(position.side_to_move, Side::Empty)))]
    #[case::no_castling_rights(|position: &Position| {
        assert_eq!(position.castling_rights, CastlingRights::NoCastling as u8);
    })]
    #[case::zero_halfmove_count(|position: &Position| assert_eq!(position.halfmove_count, 0_u8))]
    #[case::zero_fullmove_count(|position: &Position| assert_eq!(position.fullmove_count, 0_u32))]
    fn empty_position_has_expected_fields(empty_position: Position, #[case] check: fn(&Position)) {
        check(&empty_position);
    }
}
