use crate::{
    bitboard::Bitboard,
    castling_rights::CastlingRights,
    piece::{NUM_PIECES, Piece},
    side::{NUM_SIDES, Side},
    square::Square,
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

    /// Creates the standard chess starting position with White to move.
    ///
    /// Both sides have all castling rights, the halfmove clock is zero,
    /// and the fullmove number is one.
    pub(crate) fn standard() -> Self {
        use Square::*;

        let mut position = Self::empty();
        for (side, back_rank, pawns) in [
            (
                Side::White,
                [A1, B1, C1, D1, E1, F1, G1, H1],
                [A2, B2, C2, D2, E2, F2, G2, H2],
            ),
            (
                Side::Black,
                [A8, B8, C8, D8, E8, F8, G8, H8],
                [A7, B7, C7, D7, E7, F7, G7, H7],
            ),
        ] {
            let side_index = side as usize;
            let back_pieces = [
                Piece::Rook,
                Piece::Knight,
                Piece::Bishop,
                Piece::Queen,
                Piece::King,
                Piece::Bishop,
                Piece::Knight,
                Piece::Rook,
            ];
            for (piece, square) in back_pieces
                .into_iter()
                .zip(back_rank)
                .chain(pawns.into_iter().map(|square| (Piece::Pawn, square)))
            {
                position.pieces[piece as usize]
                    .set(square)
                    .expect("starting-position squares are valid");
                position.sides[side_index]
                    .set(square)
                    .expect("starting-position squares are valid");
            }
        }
        position.side_to_move = Side::White;
        position.castling_rights = CastlingRights::All as u8;
        position.fullmove_count = 1;
        position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::Piece;
    use rstest::{fixture, rstest};

    fn bitboard_from_mask(mask: u64) -> Bitboard {
        let mut board = Bitboard::empty();
        for index in 0_u8..64 {
            if mask & (1_u64 << index) != 0 {
                board |= Bitboard::try_from(Square::try_from(index).unwrap()).unwrap();
            }
        }
        board
    }

    #[rstest]
    #[case::pawn(Piece::Pawn, 0x00ff_0000_0000_ff00)]
    #[case::knight(Piece::Knight, 0x4200_0000_0000_0042)]
    #[case::bishop(Piece::Bishop, 0x2400_0000_0000_0024)]
    #[case::rook(Piece::Rook, 0x8100_0000_0000_0081)]
    #[case::queen(Piece::Queen, 0x0800_0000_0000_0008)]
    #[case::king(Piece::King, 0x1000_0000_0000_0010)]
    fn standard_position_has_expected_piece_squares(#[case] piece: Piece, #[case] mask: u64) {
        let position = Position::standard();
        assert_eq!(position.pieces[piece as usize], bitboard_from_mask(mask));
    }

    #[rstest]
    #[case::white(Side::White, 0x0000_0000_0000_ffff)]
    #[case::black(Side::Black, 0xffff_0000_0000_0000)]
    fn standard_position_has_expected_side_squares(#[case] side: Side, #[case] mask: u64) {
        let position = Position::standard();
        assert_eq!(position.sides[side as usize], bitboard_from_mask(mask));
    }

    #[test]
    fn standard_position_has_expected_fields() {
        let position = Position::standard();
        assert!(matches!(position.side_to_move, Side::White));
        assert_eq!(position.castling_rights, CastlingRights::All as u8);
        assert_eq!(position.halfmove_count, 0);
        assert_eq!(position.fullmove_count, 1);
    }

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
